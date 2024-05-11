use super::PmxModelElement;
use crate::gfx::elements::{Material, MaterialPropertyValue};
use lvl_math::{Vec3, Vec4};
use lvl_resource::{
    PmxModelMorph, PmxModelMorphKind, PmxModelMorphMaterialElement, PmxModelMorphMaterialOffsetMode,
};
use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap, HashMap},
    mem::size_of,
    num::NonZeroU64,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue,
};
use zerocopy::AsBytes;

// TODO: make engine decide the maximum morph count, not hardcoded
const MAX_MORPH_COUNT: usize = 128;

#[derive(Debug)]
pub struct Morph {
    is_dirty: AtomicBool,
    is_material_dirty: AtomicBool,
    kinds: Vec<PmxModelMorphKind>,
    name_index_map: HashMap<String, u32>,
    material_values: Vec<MaterialValue>,
    material_active_offsets: RefCell<Vec<MaterialActiveOffset>>,
    group_coefficients: Vec<HashMap<u32, f32>>,
    individual_coefficients: Vec<f32>,
    individual_coefficients_buffer: Arc<Buffer>,
}

impl Morph {
    pub fn new(
        morphs: &[PmxModelMorph],
        elements: &mut [PmxModelElement],
        device: &Device,
    ) -> Self {
        if MAX_MORPH_COUNT < morphs.len() {
            panic!("morph count is greater than the maximum morph count");
        }

        let mut group_coefficients = Vec::with_capacity(morphs.len());

        for _ in 0..morphs.len() {
            group_coefficients.push(HashMap::new());
        }

        let individual_coefficients = vec![0f32; MAX_MORPH_COUNT];
        let coefficients_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: individual_coefficients.as_bytes(),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });
        let coefficients_buffer = Arc::new(coefficients_buffer);

        let mut kinds = Vec::with_capacity(morphs.len());
        let mut name_index_map = HashMap::with_capacity(morphs.len());

        for (index, morph) in morphs.iter().enumerate() {
            kinds.push(morph.kind.clone());
            name_index_map.insert(morph.name.clone(), index as u32);
        }

        let mut material_values = Vec::with_capacity(elements.len());
        let mut material_active_offsets = Vec::with_capacity(elements.len());

        for element in elements {
            material_values.push(MaterialValue::from_material(&element.material));
            material_active_offsets.push(MaterialActiveOffset::new());

            element.material.set_property(
                "morph_coefficients",
                MaterialPropertyValue::StorageBuffer {
                    buffer: coefficients_buffer.clone(),
                    offset: 0,
                    size: NonZeroU64::new((size_of::<[f32; 128]>()) as u64).unwrap(),
                },
            );
        }

        Self {
            is_dirty: AtomicBool::new(false),
            is_material_dirty: AtomicBool::new(false),
            kinds,
            name_index_map,
            material_values,
            material_active_offsets: RefCell::new(material_active_offsets),
            group_coefficients,
            individual_coefficients,
            individual_coefficients_buffer: coefficients_buffer,
        }
    }

    pub fn set_morph(&mut self, name: &str, coefficient: f32) {
        let morph_index = match self.name_index_map.get(name) {
            Some(index) => *index,
            None => {
                return;
            }
        };

        if (self.individual_coefficients[morph_index as usize] - coefficient).abs() <= 0.001 {
            return;
        }

        self.individual_coefficients[morph_index as usize] = coefficient;
        self.is_dirty.store(true, Ordering::SeqCst);

        let mut is_material_dirty = false;

        match &self.kinds[morph_index as usize] {
            PmxModelMorphKind::Group(elements) => {
                let is_removed = coefficient.abs() <= 0.001;

                for element in elements {
                    if let PmxModelMorphKind::Group(_) = &self.kinds[element.morph_index as usize] {
                        continue;
                    }

                    if is_removed {
                        self.group_coefficients[element.morph_index as usize]
                            .remove(&(morph_index as u32));
                    } else {
                        self.group_coefficients[element.morph_index as usize]
                            .insert(morph_index as u32, element.coefficient);
                    }

                    match &self.kinds[element.morph_index as usize] {
                        PmxModelMorphKind::Material(elements) => {
                            is_material_dirty = true;

                            for element in elements {
                                self.update_material_offsets(morph_index, &element);
                            }
                        }
                        _ => {}
                    }
                }
            }
            PmxModelMorphKind::Material(elements) => {
                is_material_dirty = true;

                for element in elements {
                    self.update_material_offsets(morph_index, &element);
                }
            }
            _ => {}
        }

        if is_material_dirty {
            self.is_material_dirty.store(true, Ordering::SeqCst);
        }
    }

    fn update_material_offsets(&self, morph_index: u32, element: &PmxModelMorphMaterialElement) {
        let coefficient = self.compute_final_coefficient(morph_index);
        let is_removed = coefficient.abs() <= 0.001;
        let mut material_active_offsets = self.material_active_offsets.borrow_mut();

        match (is_removed, element.material_index) {
            (false, Some(material_index)) => {
                material_active_offsets[material_index as usize]
                    .add(morph_index as u32, MaterialOffset::from_element(&element));
            }
            (false, None) => {
                for active_offsets in material_active_offsets.iter_mut() {
                    active_offsets.add(morph_index as u32, MaterialOffset::from_element(&element));
                }
            }
            (true, Some(material_index)) => {
                material_active_offsets[material_index as usize].remove(morph_index as u32);
            }
            (true, None) => {
                for active_offsets in material_active_offsets.iter_mut() {
                    active_offsets.remove(morph_index as u32);
                }
            }
        }
    }

    pub(crate) fn update_material_values(&self, elements: &mut [PmxModelElement]) {
        if !self.is_material_dirty.load(Ordering::SeqCst) {
            return;
        }

        let material_active_offsets = self.material_active_offsets.borrow();

        for (element_index, element) in elements.iter_mut().enumerate() {
            let offsets = &material_active_offsets[element_index];

            if !offsets.is_dirty {
                continue;
            }

            let mut value = self.material_values[element_index].clone();

            for (morph_index, offset) in offsets.offsets.iter() {
                let coefficient = self.compute_final_coefficient(*morph_index);
                offset.apply(&mut value, coefficient);
            }

            value.apply(&mut element.material);
        }

        self.is_material_dirty.store(false, Ordering::SeqCst);
    }

    fn compute_final_coefficient(&self, morph_index: u32) -> f32 {
        let mut coefficient = self.individual_coefficients[morph_index as usize];

        for (_, group_coefficient) in &self.group_coefficients[morph_index as usize] {
            coefficient += group_coefficient;
        }

        coefficient
    }

    pub(crate) fn update_coefficients(&self, queue: &Queue) {
        if !self.is_dirty.load(Ordering::SeqCst) {
            return;
        }

        let mut coefficients = vec![0f32; MAX_MORPH_COUNT];

        for morph_index in 0..self.group_coefficients.len() {
            coefficients[morph_index] = self.compute_final_coefficient(morph_index as u32);
        }

        queue.write_buffer(
            &self.individual_coefficients_buffer,
            0,
            coefficients.as_bytes(),
        );
        self.is_dirty.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct MaterialValue {
    pub diffuse_color: Vec4,
    pub specular_color: Vec3,
    pub specular_strength: f32,
    pub ambient_color: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_tint_color_mul: Vec4,
    pub texture_tint_color_add: Vec4,
    pub environment_tint_color_mul: Vec4,
    pub environment_tint_color_add: Vec4,
    pub toon_tint_color_mul: Vec4,
    pub toon_tint_color_add: Vec4,
}

impl MaterialValue {
    pub fn from_material(material: &Material) -> Self {
        Self {
            diffuse_color: material
                .get_property("diffuse_color")
                .and_then(|property| property.value())
                .and_then(|value| match value {
                    MaterialPropertyValue::Vec4(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or_default(),
            specular_color: material
                .get_property("specular_color")
                .and_then(|property| property.value())
                .and_then(|value| match value {
                    MaterialPropertyValue::Vec3(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or_default(),
            specular_strength: material
                .get_property("specular_strength")
                .and_then(|property| property.value())
                .and_then(|value| match value {
                    MaterialPropertyValue::Float(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or_default(),
            ambient_color: material
                .get_property("ambient_color")
                .and_then(|property| property.value())
                .and_then(|value| match value {
                    MaterialPropertyValue::Vec3(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or_default(),
            edge_color: material
                .get_property("edge_color")
                .and_then(|property| property.value())
                .and_then(|value| match value {
                    MaterialPropertyValue::Vec4(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or_default(),
            edge_size: material
                .get_property("edge_size")
                .and_then(|property| property.value())
                .and_then(|value| match value {
                    MaterialPropertyValue::Float(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or_default(),
            texture_tint_color_mul: Vec4::ONE,
            texture_tint_color_add: Vec4::ZERO,
            environment_tint_color_mul: Vec4::ONE,
            environment_tint_color_add: Vec4::ZERO,
            toon_tint_color_mul: Vec4::ONE,
            toon_tint_color_add: Vec4::ZERO,
        }
    }

    pub fn apply(&self, material: &mut Material) {
        material.set_property(
            "diffuse_color",
            MaterialPropertyValue::Vec4(self.diffuse_color),
        );
        material.set_property(
            "specular_color",
            MaterialPropertyValue::Vec3(self.specular_color),
        );
        material.set_property(
            "specular_strength",
            MaterialPropertyValue::Float(self.specular_strength),
        );
        material.set_property(
            "ambient_color",
            MaterialPropertyValue::Vec3(self.ambient_color),
        );
        material.set_property("edge_color", MaterialPropertyValue::Vec4(self.edge_color));
        material.set_property("edge_size", MaterialPropertyValue::Float(self.edge_size));
        material.set_property(
            "texture_tint_color_mul ",
            MaterialPropertyValue::Vec4(self.texture_tint_color_mul),
        );
        material.set_property(
            "texture_tint_color_add",
            MaterialPropertyValue::Vec4(self.texture_tint_color_add),
        );
        material.set_property(
            "env_tint_color_mul",
            MaterialPropertyValue::Vec4(self.environment_tint_color_mul),
        );
        material.set_property(
            "env_tint_color_add",
            MaterialPropertyValue::Vec4(self.environment_tint_color_add),
        );
        material.set_property(
            "toon_tint_color_mul",
            MaterialPropertyValue::Vec4(self.toon_tint_color_mul),
        );
        material.set_property(
            "toon_tint_color_add",
            MaterialPropertyValue::Vec4(self.toon_tint_color_add),
        );
    }
}

#[derive(Debug)]
pub struct MaterialActiveOffset {
    pub is_dirty: bool,
    pub offsets: BTreeMap<u32, MaterialOffset>,
}

impl MaterialActiveOffset {
    pub fn new() -> Self {
        Self {
            is_dirty: false,
            offsets: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, morph_index: u32, offset: MaterialOffset) {
        match self.offsets.entry(morph_index) {
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(offset);
                self.is_dirty = true;
            }
        }
    }

    pub fn remove(&mut self, morph_index: u32) {
        match self.offsets.remove(&morph_index) {
            Some(_) => self.is_dirty = true,
            None => {}
        }
    }
}

#[derive(Debug)]
pub struct MaterialOffset {
    pub offset_mode: PmxModelMorphMaterialOffsetMode,
    pub diffuse_color: Vec4,
    pub specular_color: Vec3,
    pub specular_strength: f32,
    pub ambient_color: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_tint_color: Vec4,
    pub environment_tint_color: Vec4,
    pub toon_tint_color: Vec4,
}

impl MaterialOffset {
    pub fn from_element(element: &PmxModelMorphMaterialElement) -> Self {
        Self {
            offset_mode: element.offset_mode,
            diffuse_color: element.diffuse_color,
            specular_color: element.specular_color,
            specular_strength: element.specular_strength,
            ambient_color: element.ambient_color,
            edge_color: element.edge_color,
            edge_size: element.edge_size,
            texture_tint_color: element.texture_tint_color,
            environment_tint_color: element.environment_tint_color,
            toon_tint_color: element.toon_tint_color,
        }
    }

    pub fn apply(&self, value: &mut MaterialValue, weight: f32) {
        match self.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                value.diffuse_color = Vec4::lerp_unclamped(
                    value.diffuse_color,
                    value.diffuse_color * self.diffuse_color,
                    weight,
                );
                value.specular_color = Vec3::lerp_unclamped(
                    value.specular_color,
                    value.specular_color * self.specular_color,
                    weight,
                );
                value.specular_strength = lerp_unclamped_f32(
                    value.specular_strength,
                    value.specular_strength * self.specular_strength,
                    weight,
                );
                value.ambient_color = Vec3::lerp_unclamped(
                    value.ambient_color,
                    value.ambient_color * self.ambient_color,
                    weight,
                );
                value.edge_color = Vec4::lerp_unclamped(
                    value.edge_color,
                    value.edge_color * self.edge_color,
                    weight,
                );
                value.edge_size =
                    lerp_unclamped_f32(value.edge_size, value.edge_size * self.edge_size, weight);
                value.texture_tint_color_mul = Vec4::lerp_unclamped(
                    value.texture_tint_color_mul,
                    value.texture_tint_color_mul * self.texture_tint_color,
                    weight,
                );
                value.environment_tint_color_mul = Vec4::lerp_unclamped(
                    value.environment_tint_color_mul,
                    value.environment_tint_color_mul * self.environment_tint_color,
                    weight,
                );
                value.toon_tint_color_mul = Vec4::lerp_unclamped(
                    value.toon_tint_color_mul,
                    value.toon_tint_color_mul * self.toon_tint_color,
                    weight,
                );
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                value.diffuse_color += self.diffuse_color * weight;
                value.specular_color += self.specular_color * weight;
                value.specular_strength += self.specular_strength * weight;
                value.ambient_color += self.ambient_color * weight;
                value.edge_color += self.edge_color * weight;
                value.edge_size += self.edge_size * weight;
                value.texture_tint_color_add += self.texture_tint_color * weight;
                value.environment_tint_color_add += self.environment_tint_color * weight;
                value.toon_tint_color_add += self.toon_tint_color * weight;
            }
        }
    }
}

fn lerp_unclamped_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

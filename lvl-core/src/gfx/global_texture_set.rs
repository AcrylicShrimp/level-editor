use wgpu::{
    Device, Extent3d, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

pub struct GlobalTextureSet {
    pub msaa_sample_count: u32,
    pub color: Option<TextureSet>,
    pub depth_stencil: TextureSet,
}

impl GlobalTextureSet {
    pub(crate) fn new(
        device: &Device,
        size: PhysicalSize<u32>,
        color_texture_format: TextureFormat,
        msaa_sample_count: u32,
    ) -> Self {
        Self {
            msaa_sample_count,
            color: if msaa_sample_count == 1 {
                None
            } else {
                Some(TextureSet::new(
                    device,
                    "color",
                    size,
                    color_texture_format,
                    TextureUsages::RENDER_ATTACHMENT,
                    msaa_sample_count,
                ))
            },
            depth_stencil: TextureSet::new(
                device,
                "depth stencil",
                size,
                TextureFormat::Depth32Float,
                TextureUsages::RENDER_ATTACHMENT,
                msaa_sample_count,
            ),
        }
    }

    pub(crate) fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        if let Some(color) = &mut self.color {
            color.resize(device, size);
        }

        self.depth_stencil.resize(device, size);
    }
}

pub struct TextureSet {
    pub texture: Texture,
    pub texture_view: TextureView,
    name: String,
    format: TextureFormat,
    usage: TextureUsages,
    sample_count: u32,
}

impl TextureSet {
    fn new(
        device: &Device,
        name: impl Into<String>,
        size: PhysicalSize<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        sample_count: u32,
    ) -> Self {
        let name = name.into();
        let (texture, texture_view) =
            Self::create_texture_and_view(device, name.as_str(), size, format, usage, sample_count);

        Self {
            texture,
            texture_view,
            name,
            format,
            usage,
            sample_count,
        }
    }

    fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        let (texture, texture_view) = Self::create_texture_and_view(
            device,
            self.name.as_str(),
            size,
            self.format,
            self.usage,
            self.sample_count,
        );

        self.texture = texture;
        self.texture_view = texture_view;
    }

    fn create_texture_and_view(
        device: &Device,
        name: &str,
        size: PhysicalSize<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        sample_count: u32,
    ) -> (Texture, TextureView) {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&name),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some(&format!("{} view", name)),
            format: None,
            dimension: None,
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        (texture, texture_view)
    }
}

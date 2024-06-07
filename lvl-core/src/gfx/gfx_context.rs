use super::{Frame, GlobalTextureSet, PerFrameBufferPool, UniformBindGroupProvider};
use std::cell::RefCell;
use thiserror::Error;
use wgpu::{
    Adapter, Backend, Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, DeviceType,
    Features, Instance, InstanceDescriptor, MaintainBase, PresentMode, Queue, Surface,
    SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureUsages,
};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Error, Debug)]
pub enum GfxContextCreationError {
    #[error("no adapter found")]
    AdapterNotFound,
    #[error("surface not supported")]
    SurfaceNotSupported,
    #[error("failed to obtain device: {0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("failed to create surface: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
}

pub struct GfxContext<'window> {
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'window>,
    pub surface_config: RefCell<SurfaceConfiguration>,
    pub global_texture_set: RefCell<GlobalTextureSet>,
    pub per_frame_buffer_pool: PerFrameBufferPool,
    pub uniform_bind_group_provider: UniformBindGroupProvider,
}

impl<'window> GfxContext<'window> {
    pub(crate) async fn new(
        window: &'window Window,
        vsync: bool,
    ) -> Result<Self, GfxContextCreationError> {
        let instance = Instance::new(InstanceDescriptor::default());
        let surface = instance.create_surface(window)?;
        let adapters = instance.enumerate_adapters(Backends::all());
        let adapter = match select_adapter(&surface, &adapters) {
            Some(adapter_index) => &adapters[adapter_index],
            None => return Err(GfxContextCreationError::AdapterNotFound),
        };

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::CLEAR_TEXTURE,
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await?;

        let adapter_surface_caps = surface.get_capabilities(adapter);
        let preferred_format = match adapter_surface_caps.formats.first() {
            Some(format) => *format,
            None => return Err(GfxContextCreationError::SurfaceNotSupported),
        };
        let preferred_alpha_mode = match adapter_surface_caps.alpha_modes.first() {
            Some(mode) => *mode,
            None => return Err(GfxContextCreationError::SurfaceNotSupported),
        };

        let window_inner_size = window.inner_size();
        let surface_config = RefCell::new(SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: preferred_format,
            width: window_inner_size.width,
            height: window_inner_size.height,
            present_mode: if vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: preferred_alpha_mode,
            view_formats: vec![],
        });
        surface.configure(&device, &surface_config.borrow());

        let global_texture_set = RefCell::new(GlobalTextureSet::new(&device, window_inner_size));
        let per_frame_buffer_pool = PerFrameBufferPool::new();
        let uniform_bind_group_provider = UniformBindGroupProvider::new(&device);

        Ok(GfxContext {
            instance,
            device,
            queue,
            surface,
            surface_config,
            global_texture_set,
            per_frame_buffer_pool,
            uniform_bind_group_provider,
        })
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        let mut surface_config = self.surface_config.borrow_mut();
        surface_config.width = size.width;
        surface_config.height = size.height;

        self.surface.configure(&self.device, &surface_config);
        self.global_texture_set
            .borrow_mut()
            .resize(&self.device, size);
    }

    pub fn obtain_surface_view(&self) -> Result<SurfaceTexture, SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn begin_frame(&self) -> Frame {
        self.per_frame_buffer_pool.reset();

        let cmd_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("[GfxContext] begin_frame"),
            });
        Frame::new(cmd_encoder)
    }

    pub fn end_frame(&self, frame: Frame) {
        self.queue.submit(std::iter::once(frame.finish()));
        self.device.poll(MaintainBase::Wait);
    }
}

fn select_adapter(surface: &Surface, adapters: impl AsRef<[Adapter]>) -> Option<usize> {
    let adapters = adapters
        .as_ref()
        .iter()
        .filter(|adapter| !surface.get_capabilities(adapter).formats.is_empty())
        .collect::<Vec<_>>();

    if adapters.is_empty() {
        return None;
    }

    let mut scores = adapters.iter().map(|_| 0).collect::<Vec<_>>();

    for (index, adapter) in adapters.iter().enumerate() {
        if surface.get_capabilities(adapter).formats.is_empty() {
            continue;
        }

        let info = adapter.get_info();
        let device_score = match info.device_type {
            DeviceType::Other => 0,
            DeviceType::IntegratedGpu => 10,
            DeviceType::DiscreteGpu => 20,
            DeviceType::VirtualGpu => 5,
            DeviceType::Cpu => -10,
        };
        let backend_score = match info.backend {
            // The Vulkan is available with other backends simultaneously on some platforms.
            // Because the dedicated backends are preferred over the Vulkan, we set the score of the Vulkan slightly lower than others.
            Backend::Metal => 2,
            Backend::Dx12 => 2,
            Backend::Vulkan => 1,
            _ => 0,
        };
        scores[index] += device_score + backend_score;
    }

    scores
        .into_iter()
        .enumerate()
        .max_by_key(|(_, score)| *score)
        .map(|(index, _)| index)
}

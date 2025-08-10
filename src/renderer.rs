use winit::window::Window;

/// Provides the output display to the window.
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&Default::default());

        let adapter = futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })).expect("Failed to find appropriate adapter");

        let (device, queue) = futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            ..Default::default()
        })).expect("Failed to create device");

        Self {
            instance,
            adapter,
            device,
            queue
        }
    }

    pub fn create_surface(&self, window: std::sync::Arc<Window>) -> Surface {
        let size = window.inner_size();
        let surface = self.instance.create_surface(window.clone()).expect("Failed to create surface");
        let surface_config = surface.get_default_config(&self.adapter, size.width, size.height).expect("Could not get default surface config");
        surface.configure(&self.device, &surface_config);

        Surface {
            surface, surface_config
        }
    }

    pub fn resize_surface(&self, surface: &mut Surface, width: u32, height: u32) {
        surface.resize(width, height, &self.device);
    }
}

pub struct Surface {
    surface:        wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

impl Surface {
    fn resize(&mut self, width: u32, height: u32, device: &wgpu::Device) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(device, &self.surface_config);
    }
}
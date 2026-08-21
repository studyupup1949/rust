use acute_ecs::prelude::*;
use acute_window::winit::window::Window;
use std::ops::Deref;

pub struct Renderer {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub swap_chain: wgpu::SwapChain,
    pub window: Window,
}

impl Renderer {
    pub async fn new(
        resources: &mut Resources,
        window: Window,
    ) -> Self {

        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        resources.insert(sc_desc);

        Self {
            adapter,
            device,
            queue,
            surface,
            swap_chain,
            window
        }
    }
}
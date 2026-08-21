//! `GpuBackend` — wgpu compute-shader implementation of [`ArithmeticBackend`].
//!
//! wgpu is always compiled in (not feature-gated). At startup [`GpuBackend::try_init`]
//! probes for a compatible adapter; if none is found it returns `Err` and the
//! [`crate::backend::Executor`] transparently falls back to the CPU backend.
//!
//! Each shader thread handles one `(batch_item × channel)` pair. The buffer
//! layout is identical to [`crate::batch::RnsBatch`], so there is no reformatting
//! on upload beyond the `u64 -> u32` narrowing (safe because all moduli `< 2^16`).

use bytemuck::{Pod, Zeroable};
use num_bigint::BigUint;
use wgpu::util::DeviceExt;

use crate::backend::ArithmeticBackend;
use crate::batch::RnsBatch;
use crate::rns::garner_crt;

/// Errors that can occur while bringing up the GPU backend.
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("no compatible GPU adapter found")]
    NoAdapter,
    #[error("failed to acquire GPU device: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    batch_size: u32,
    n_channels: u32,
    _pad: [u32; 2], // pad to 16 bytes for std140 uniform layout
}

/// GPU backend holding a device, queue, and the pre-built compute pipelines.
pub struct GpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layout: wgpu::BindGroupLayout,
    add_pipeline: wgpu::ComputePipeline,
    mul_pipeline: wgpu::ComputePipeline,
    adapter_info: wgpu::AdapterInfo,
}

impl GpuBackend {
    /// Probe for a GPU and build the pipelines. Blocks on async init.
    pub fn try_init() -> Result<Self, GpuError> {
        pollster::block_on(Self::try_init_async())
    }

    async fn try_init_async() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("adele-ring-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let add_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/rns_add.wgsl"));
        let mul_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/rns_mul.wgsl"));

        let bind_group_layout = Self::make_bind_group_layout(&device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("adele-ring-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let add_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rns-add"),
            layout: Some(&pipeline_layout),
            module: &add_shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        let mul_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rns-mul"),
            layout: Some(&pipeline_layout),
            module: &mul_shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            bind_group_layout,
            add_pipeline,
            mul_pipeline,
            adapter_info,
        })
    }

    /// Human-readable adapter name (e.g. "NVIDIA GeForce RTX 4080").
    pub fn adapter_name(&self) -> &str {
        &self.adapter_info.name
    }

    fn make_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let storage = |read_only: bool| wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("adele-ring-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: storage(true),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: storage(true),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: storage(true),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: storage(false),
                    count: None,
                },
            ],
        })
    }

    fn run_pipeline(
        &self,
        pipeline: &wgpu::ComputePipeline,
        a: &RnsBatch,
        b: &RnsBatch,
    ) -> RnsBatch {
        let k = a.channels.len();
        let b_size = a.batch_size;
        let n_elems = b_size * k;
        let byte_len = (n_elems * std::mem::size_of::<u32>()) as u64;

        let params = Params {
            batch_size: b_size as u32,
            n_channels: k as u32,
            _pad: [0, 0],
        };
        let params_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let moduli_u32: Vec<u32> = a.channels.moduli().iter().map(|&m| m as u32).collect();
        let moduli_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("moduli"),
                contents: bytemuck::cast_slice(&moduli_u32),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let a_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("a"),
                contents: &a.as_u32_bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let b_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("b"),
                contents: &b.as_u32_bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let out_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out"),
            size: byte_len,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: byte_len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("adele-ring-bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: moduli_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: a_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: b_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: out_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("rns-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                (b_size as u32).div_ceil(16),
                (k as u32).div_ceil(16),
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&out_buf, 0, &staging_buf, 0, byte_len);
        self.queue.submit([encoder.finish()]);

        let slice = staging_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .expect("map_async channel closed")
            .expect("buffer map failed");

        let data = slice.get_mapped_range();
        let values: &[u32] = bytemuck::cast_slice(&data);
        let result = RnsBatch::from_u32(values, b_size, a.channels.clone());
        drop(data);
        staging_buf.unmap();
        result
    }
}

impl ArithmeticBackend for GpuBackend {
    fn batch_rns_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.run_pipeline(&self.add_pipeline, a, b)
    }

    fn batch_rns_mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.run_pipeline(&self.mul_pipeline, a, b)
    }

    fn batch_crt(&self, batch: &RnsBatch) -> Vec<BigUint> {
        // CRT (Garner) is sequential; do it on the CPU regardless of backend.
        let k = batch.channels.len();
        let moduli = batch.channels.moduli();
        (0..batch.batch_size)
            .map(|b| garner_crt(&batch.data[b * k..(b + 1) * k], moduli))
            .collect()
    }

    fn name(&self) -> &'static str {
        "gpu-wgpu"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rns::{Channels, RnsInt};

    #[test]
    fn gpu_matches_cpu_when_available() {
        let gpu = match GpuBackend::try_init() {
            Ok(g) => g,
            Err(_) => return, // no GPU on this machine; skip
        };
        let ch = Channels::standard(32);
        let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, ch.clone()); 256]);
        let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, ch.clone()); 256]);

        let cpu = crate::cpu::CpuBackend::new();
        assert_eq!(
            cpu.batch_rns_add(&a, &b).data,
            gpu.batch_rns_add(&a, &b).data
        );
        assert_eq!(
            cpu.batch_rns_mul(&a, &b).data,
            gpu.batch_rns_mul(&a, &b).data
        );
    }
}

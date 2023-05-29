pub mod convolve;
pub mod trace;

use crate::gpu::trace::chunk::Chunk;
use crossbeam::channel::Receiver;
use glam::{IVec2, UVec2};
use image::ImageBuffer;
use wgpu::{
    Buffer, Extent3d, RenderPipeline, ShaderModuleDescriptor, TextureDescriptor, TextureDimension,
    TextureFormat, TextureView,
};

use crate::gpu::trace::world::{TraceState, WorldChange};

pub enum InterfaceToGpuMessage {
    WorldChange(WorldChange),
    RunDebugRender,
}
pub struct DebugRenderer {
    render_pipeline: RenderPipeline,
    out_buf: Buffer,
    texture: wgpu::Texture,
    device: wgpu::Device,
    queue: wgpu::Queue,
    world_state: TraceState,
}
impl DebugRenderer {
    pub async fn new() -> Self {
        env_logger::init();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let tex_desc = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        };
        let texture = device.create_texture(&tex_desc);
        let output_buffer_size =
            (std::mem::size_of::<u32>() as u32 * 1920 * 1080) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let out_buf = device.create_buffer(&output_buffer_desc);
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shared.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[],           // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: texture.format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,                  //Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });
        let world_state = TraceState::new(&device);
        Self {
            render_pipeline,
            out_buf,
            texture,
            device,
            queue,
            world_state,
        }
    }

    pub async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let view = self.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        }
        self.world_state.apply_diffs(&self.device, &mut encoder);
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.out_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::mem::size_of::<u32>() as u32 * 1920),
                    rows_per_image: Some(1080),
                },
            },
            Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        {
            let sliced = self.out_buf.slice(..);
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            sliced.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device.poll(wgpu::Maintain::Wait);
            rx.receive().await.unwrap().unwrap();
            let data = sliced.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(1920, 1080, data).unwrap();
            let _ = std::fs::remove_file("image.png");
            buffer.save("image.png").expect("failed to save image");
        }
        self.out_buf.unmap();
        Ok(())
    }
    pub fn process(&mut self, msg: InterfaceToGpuMessage) {
        match msg {
            InterfaceToGpuMessage::WorldChange(change) => {
                self.world_state.queue_diff(change);
            }
            InterfaceToGpuMessage::RunDebugRender => {
                println!("running render!");
                pollster::block_on(self.render()).unwrap();
            }
        }
    }
}

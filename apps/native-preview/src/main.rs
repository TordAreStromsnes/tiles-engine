use std::{borrow::Cow, env, error::Error, fmt, time::Instant};

use bytemuck::{Pod, Zeroable};
use tiles_renderer::{default_preview_scene, preview_sprite_batch, PreviewScene};
use wgpu::util::DeviceExt;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const WINDOW_TITLE: &str = "Tiles Engine Native Preview";
const DEFAULT_WINDOW_WIDTH: f64 = 960.0;
const DEFAULT_WINDOW_HEIGHT: f64 = 640.0;
const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
    },
    Vertex {
        position: [0.5, -0.5],
    },
    Vertex {
        position: [0.5, 0.5],
    },
    Vertex {
        position: [-0.5, 0.5],
    },
];
const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
const MAX_INSTANCES: u64 = 512;
const SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) offset: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position * vertex.scale + vertex.offset, 0.0, 1.0);
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(fragment: VertexOut) -> @location(0) vec4<f32> {
    return fragment.color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceRaw {
    offset: [f32; 2],
    scale: [f32; 2],
    color: [f32; 4],
}

impl InstanceRaw {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x4
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(Debug)]
enum PreviewError {
    AdapterUnavailable,
    CreateSurface(wgpu::CreateSurfaceError),
    RequestDevice(wgpu::RequestDeviceError),
    Window(winit::error::OsError),
    EventLoop(winit::error::EventLoopError),
}

impl fmt::Display for PreviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdapterUnavailable => write!(formatter, "no compatible GPU adapter was found"),
            Self::CreateSurface(error) => {
                write!(formatter, "failed to create GPU surface: {error}")
            }
            Self::RequestDevice(error) => {
                write!(formatter, "failed to request GPU device: {error}")
            }
            Self::Window(error) => write!(formatter, "failed to create native window: {error}"),
            Self::EventLoop(error) => write!(formatter, "native event loop failed: {error}"),
        }
    }
}

impl Error for PreviewError {}

impl From<winit::error::OsError> for PreviewError {
    fn from(error: winit::error::OsError) -> Self {
        Self::Window(error)
    }
}

impl From<winit::error::EventLoopError> for PreviewError {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self::EventLoop(error)
    }
}

impl From<wgpu::CreateSurfaceError> for PreviewError {
    fn from(error: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurface(error)
    }
}

impl From<wgpu::RequestDeviceError> for PreviewError {
    fn from(error: wgpu::RequestDeviceError) -> Self {
        Self::RequestDevice(error)
    }
}

struct PreviewRenderer<'window> {
    scene: PreviewScene,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    index_count: u32,
}

impl<'window> PreviewRenderer<'window> {
    async fn new(window: &'window Window, scene: PreviewScene) -> Result<Self, PreviewError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(PreviewError::AdapterUnavailable)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("tiles-preview-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let alpha_mode = capabilities.alpha_modes[0];
        let config = surface_config(size, format, present_mode, alpha_mode);
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tiles-preview-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("tiles-preview-pipeline-layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tiles-preview-render-pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), InstanceRaw::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tiles-preview-quad-vertices"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tiles-preview-quad-indices"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tiles-preview-instances"),
            size: MAX_INSTANCES * std::mem::size_of::<InstanceRaw>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            scene,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            index_count: QUAD_INDICES.len() as u32,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self, elapsed_seconds: f32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let instances = build_instances(&self.scene, elapsed_seconds);
        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tiles-preview-render-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tiles-preview-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.045,
                            g: 0.055,
                            b: 0.075,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..self.index_count, 0, 0..instances.len() as u32);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn main() -> Result<(), PreviewError> {
    let frame_limit = parse_frame_limit();
    pollster::block_on(run(frame_limit))
}

async fn run(frame_limit: Option<u32>) -> Result<(), PreviewError> {
    let renderer = tiles_renderer::native_renderer_plan();
    let runtime = tiles_runtime::native_runtime_boundary();
    let scene = default_preview_scene();

    println!("Tiles Engine native preview");
    println!("renderer: {}", renderer.backend_summary());
    println!("preview: {}", renderer.preview_strategy);
    println!("runtime: {}", runtime.game_loop_owner);
    println!(
        "scene: {}x{} tile grid with animated sprite",
        scene.grid.columns, scene.grid.rows
    );

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT,
        ))
        .build(&event_loop)?;
    let window: &'static Window = Box::leak(Box::new(window));
    let mut renderer = PreviewRenderer::new(window, scene).await?;
    let start_time = Instant::now();
    let mut rendered_frames = 0_u32;

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(size) => renderer.resize(size),
                WindowEvent::RedrawRequested => {
                    let elapsed_seconds = start_time.elapsed().as_secs_f32();

                    match renderer.render(elapsed_seconds) {
                        Ok(()) => {
                            rendered_frames += 1;

                            if frame_limit.is_some_and(|limit| rendered_frames >= limit) {
                                target.exit();
                            }
                        }
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(renderer.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(wgpu::SurfaceError::Timeout) => {}
                    }
                }
                _ => {}
            },
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    })?;

    Ok(())
}

fn surface_config(
    size: PhysicalSize<u32>,
    format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    alpha_mode: wgpu::CompositeAlphaMode,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

fn build_instances(scene: &PreviewScene, elapsed_seconds: f32) -> Vec<InstanceRaw> {
    let batch = preview_sprite_batch(scene, elapsed_seconds);

    batch
        .sorted_instances()
        .into_iter()
        .map(|instance| InstanceRaw {
            offset: instance.position,
            scale: [
                if instance.flip_x {
                    -instance.size[0]
                } else {
                    instance.size[0]
                },
                if instance.flip_y {
                    -instance.size[1]
                } else {
                    instance.size[1]
                },
            ],
            color: instance.tint,
        })
        .collect()
}

fn parse_frame_limit() -> Option<u32> {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke-test" => return Some(3),
            "--frames" => {
                if let Some(value) = args.next().and_then(|value| value.parse().ok()) {
                    return Some(value);
                }
            }
            _ => {}
        }
    }

    None
}

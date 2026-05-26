use std::{borrow::Cow, collections::HashMap, env, error::Error, fmt, time::Instant};

use bytemuck::{Pod, Zeroable};
use tiles_renderer::{
    default_preview_scene, preview_camera, preview_editor_overlay_batch, preview_sprite_batch,
    preview_texture_atlases, Camera2d, PreviewScene, SpriteBatch, TextureAtlas, TextureFilterMode,
    TextureRect,
};
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
    @location(4) uv_origin: vec2<f32>,
    @location(5) uv_size: vec2<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position * vertex.scale + vertex.offset, 0.0, 1.0);
    out.color = vertex.color;
    out.uv = (vertex.position + vec2<f32>(0.5, 0.5)) * vertex.uv_size + vertex.uv_origin;
    return out;
}

@fragment
fn fs_main(fragment: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(atlas_texture, atlas_sampler, fragment.uv) * fragment.color;
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
    uv_origin: [f32; 2],
    uv_size: [f32; 2],
}

impl InstanceRaw {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x4,
        4 => Float32x2,
        5 => Float32x2
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
    camera: Camera2d,
    render_pipeline: wgpu::RenderPipeline,
    atlases: HashMap<String, TextureAtlas>,
    atlas_bind_groups: HashMap<String, wgpu::BindGroup>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    overlay_instance_buffer: wgpu::Buffer,
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
        let camera = camera_for_surface(&scene, size);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tiles-preview-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("tiles-preview-atlas-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let atlases = preview_texture_atlases();
        let atlas_bind_groups = atlases
            .iter()
            .map(|atlas| {
                (
                    atlas.id.clone(),
                    create_preview_atlas_bind_group(
                        &device,
                        &queue,
                        &atlas_bind_group_layout,
                        atlas,
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        let atlases = atlases
            .into_iter()
            .map(|atlas| (atlas.id.clone(), atlas))
            .collect::<HashMap<_, _>>();
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("tiles-preview-pipeline-layout"),
                bind_group_layouts: &[&atlas_bind_group_layout],
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
        let overlay_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tiles-preview-overlay-instances"),
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
            camera,
            render_pipeline,
            atlases,
            atlas_bind_groups,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            overlay_instance_buffer,
            index_count: QUAD_INDICES.len() as u32,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;
        self.camera = camera_for_surface(&self.scene, size);
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self, elapsed_seconds: f32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let scene_instances =
            build_instances(&self.scene, &self.camera, elapsed_seconds, &self.atlases);
        let overlay_instances =
            build_overlay_instances(&self.scene, &self.camera, elapsed_seconds, &self.atlases);
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&scene_instances.instances),
        );
        self.queue.write_buffer(
            &self.overlay_instance_buffer,
            0,
            bytemuck::cast_slice(&overlay_instances.instances),
        );

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
            for group in &scene_instances.groups {
                pass.set_bind_group(0, self.atlas_bind_group(&group.atlas_id), &[]);
                pass.draw_indexed(
                    0..self.index_count,
                    0,
                    group.start_instance..group.start_instance + group.instance_count,
                );
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tiles-preview-editor-overlay-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.overlay_instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            for group in &overlay_instances.groups {
                pass.set_bind_group(0, self.atlas_bind_group(&group.atlas_id), &[]);
                pass.draw_indexed(
                    0..self.index_count,
                    0,
                    group.start_instance..group.start_instance + group.instance_count,
                );
            }
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    fn atlas_bind_group(&self, atlas_id: &str) -> &wgpu::BindGroup {
        self.atlas_bind_groups
            .get(atlas_id)
            .expect("preview atlas bind group should exist")
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
    let camera = preview_camera(&scene);

    println!("Tiles Engine native preview");
    println!("renderer: {}", renderer.backend_summary());
    println!("preview: {}", renderer.preview_strategy);
    println!("runtime: {}", runtime.game_loop_owner);
    println!(
        "scene: {}x{} tile grid with animated sprite",
        scene.grid.columns, scene.grid.rows
    );
    println!(
        "camera: {:.2}x{:.2} world viewport, zoom {:.2}",
        camera.viewport_size[0], camera.viewport_size[1], camera.zoom
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

fn build_instances(
    scene: &PreviewScene,
    camera: &Camera2d,
    elapsed_seconds: f32,
    atlases: &HashMap<String, TextureAtlas>,
) -> RenderInstanceGroups {
    build_batch_instances(
        &preview_sprite_batch(scene, elapsed_seconds),
        camera,
        atlases,
    )
}

fn build_overlay_instances(
    scene: &PreviewScene,
    camera: &Camera2d,
    elapsed_seconds: f32,
    atlases: &HashMap<String, TextureAtlas>,
) -> RenderInstanceGroups {
    build_batch_instances(
        &preview_editor_overlay_batch(scene, elapsed_seconds),
        camera,
        atlases,
    )
}

struct RenderInstanceGroups {
    instances: Vec<InstanceRaw>,
    groups: Vec<RenderInstanceGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderInstanceGroup {
    atlas_id: String,
    start_instance: u32,
    instance_count: u32,
}

fn build_batch_instances(
    batch: &SpriteBatch,
    camera: &Camera2d,
    atlases: &HashMap<String, TextureAtlas>,
) -> RenderInstanceGroups {
    let mut instances = Vec::new();
    let mut groups = Vec::new();

    for group in batch.atlas_groups_in_draw_order() {
        let atlas = atlases
            .get(&group.atlas_id)
            .expect("preview atlas metadata should exist");
        let start_instance = instances.len() as u32;

        for instance in group.instances {
            let clip_size = camera.world_size_to_clip(instance.size);

            instances.push(InstanceRaw {
                offset: camera.world_to_clip(instance.position),
                scale: [
                    if instance.flip_x {
                        -clip_size[0]
                    } else {
                        clip_size[0]
                    },
                    if instance.flip_y {
                        -clip_size[1]
                    } else {
                        clip_size[1]
                    },
                ],
                color: instance.tint,
                uv_origin: uv_origin(instance.source.source_rect, &atlas),
                uv_size: uv_size(instance.source.source_rect, &atlas),
            });
        }

        groups.push(RenderInstanceGroup {
            atlas_id: group.atlas_id,
            start_instance,
            instance_count: instances.len() as u32 - start_instance,
        });
    }

    RenderInstanceGroups { instances, groups }
}

fn camera_for_surface(scene: &PreviewScene, size: PhysicalSize<u32>) -> Camera2d {
    let mut camera = preview_camera(scene);
    let width = size.width.max(1) as f32;
    let height = size.height.max(1) as f32;
    camera.viewport_size[0] = camera.viewport_size[1] * (width / height);
    camera
}

fn create_preview_atlas_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    atlas: &TextureAtlas,
) -> wgpu::BindGroup {
    let size = wgpu::Extent3d {
        width: atlas.size.width,
        height: atlas.size.height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tiles-preview-generated-atlas"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let pixels = preview_atlas_pixels(atlas);
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * atlas.size.width),
            rows_per_image: Some(atlas.size.height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("tiles-preview-atlas-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu_filter_mode(atlas.sampling.magnify_filter),
        min_filter: wgpu_filter_mode(atlas.sampling.minify_filter),
        mipmap_filter: wgpu_filter_mode(atlas.sampling.mipmap_filter),
        ..Default::default()
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tiles-preview-atlas-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}

fn wgpu_filter_mode(filter: TextureFilterMode) -> wgpu::FilterMode {
    match filter {
        TextureFilterMode::Nearest => wgpu::FilterMode::Nearest,
        TextureFilterMode::Linear => wgpu::FilterMode::Linear,
    }
}

fn preview_atlas_pixels(atlas: &TextureAtlas) -> Vec<u8> {
    let mut pixels = vec![255; atlas.size.width as usize * atlas.size.height as usize * 4];
    let colors = [
        ("tile.checker.a", [215, 242, 206, 255]),
        ("tile.checker.b", [152, 215, 184, 255]),
        ("sprite.hero.placeholder", [245, 104, 78, 255]),
        ("overlay.selection", [255, 255, 255, 255]),
    ];

    for sprite in &atlas.sprites {
        let color = colors
            .iter()
            .find_map(|(id, color)| (*id == sprite.id).then_some(*color))
            .unwrap_or([255, 255, 255, 255]);

        for y in sprite.source_rect.y..sprite.source_rect.y + sprite.source_rect.height {
            for x in sprite.source_rect.x..sprite.source_rect.x + sprite.source_rect.width {
                let index = ((y * atlas.size.width + x) * 4) as usize;
                pixels[index..index + 4].copy_from_slice(&color);
            }
        }
    }

    pixels
}

fn uv_origin(rect: Option<TextureRect>, atlas: &TextureAtlas) -> [f32; 2] {
    let Some(rect) = rect else {
        return [0.0, 0.0];
    };

    [
        rect.x as f32 / atlas.size.width as f32,
        rect.y as f32 / atlas.size.height as f32,
    ]
}

fn uv_size(rect: Option<TextureRect>, atlas: &TextureAtlas) -> [f32; 2] {
    let Some(rect) = rect else {
        return [1.0, 1.0];
    };

    [
        rect.width as f32 / atlas.size.width as f32,
        rect.height as f32 / atlas.size.height as f32,
    ]
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

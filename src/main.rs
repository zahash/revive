use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Face, Features,
    FragmentState, FrontFace, Instance, Limits, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages,
    TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Copy)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: [Vertex; 5] = [
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [1.0, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.0, 0.1, 0.0],
    }, // E
];

const INDICES: [u16; 9] = [0, 1, 4, 1, 2, 4, 2, 3, 4];

pub struct State<'window> {
    pub surface: Surface<'window>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl<'window> State<'window> {
    pub async fn new(window: &'window Window) -> Self {
        // Its main purpose is to create Adapters and Surfaces.
        let instance = Instance::default();

        // The surface is the part of the window that we draw to
        // SAFETY : window lives atleast as longs surface.
        let surface = instance.create_surface(window).unwrap();

        // The adapter is a handle for our actual graphics card
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let PhysicalSize { width, height } = window.inner_size();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_capabilities
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_capabilities.formats[0]),
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: BufferUsages::INDEX,
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    // attributes: &vertex_attr_array![0 => Float32x3, 1=> Float32x3],
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: std::mem::size_of::<[f32; 3]>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: FrontFace::Ccw, // 2.
                cull_mode: Some(Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None, // 5.
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 1.0,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

        drop(render_pass);

        self.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.config.width,
            height: self.config.height,
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(&window).await;

    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => elwt.exit(),
                WindowEvent::Resized(size) => state.resize(size),
                WindowEvent::RedrawRequested => match state.render() {
                    Ok(_) => {}
                    Err(SurfaceError::Lost) => state.resize(state.size()),
                    Err(SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("{:?}", e),
                },
                _ => {}
            },

            _ => {}
        })
        .unwrap();
}

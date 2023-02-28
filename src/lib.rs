use std::vec;

use bytemuck::{Pod, Zeroable};
use wgpu::{Backends, util::DeviceExt};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};


pub struct App {
    state: State,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    colour: [f32; 3],
}

impl Vertex {
    fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as u64,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 1,
                },
            ],
            step_mode: wgpu::VertexStepMode::Vertex,
        }
    }
}

const VERTICES: &[Vertex; 4] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        colour: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        colour: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        colour: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        colour: [0.0, 1.0, 0.0],
    },
];

const INDICES: &[u16; 6] = &[0, 1, 2, 0, 2, 3];

impl App {
    pub async fn run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Sexy Little Engine")
            .with_inner_size(PhysicalSize::new(800, 800))
            .build(&event_loop)
            .unwrap();

        let mut app = App::new(window).await;

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == app.state.window.id() => {
                if !app.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode:
                                        Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size) => app.state.resize(*size),
                        _ => (),
                    }
                }
            }
            Event::RedrawRequested(window_id)
                if window_id == app.state.window.id() =>
            {
                app.update();
                match app.render() {
                    Ok(_) => {}

                    Err(wgpu::SurfaceError::Lost) => {
                        app.state.resize(app.state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit
                    }
                    Err(e) => {
                        eprintln!("{e:?}")
                    }
                }
            }
            Event::MainEventsCleared => {
                app.state.window.request_redraw();
            }
            _ => (),
        })
    }

    pub async fn new(window: Window) -> Self {
        let state = State::new(window).await;

        Self { state }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // nothing
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // nothing
        let output = self.state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.state.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            },
        );

        // render pass in its own block
        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear({
                                    wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 0.0,
                                    }
                                }),
                                store: true,
                            },
                            resolve_target: None,
                        },
                    )],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.state.pipeline);

                render_pass.set_vertex_buffer(0, self.state.v_buff.slice(..));
        
                render_pass.set_index_buffer(self.state.i_buff.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..6, 0, 0..1);
        
        
        
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct State {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    v_buff: wgpu::Buffer,
    i_buff: wgpu::Buffer,
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
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
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            alpha_mode: surface_caps.alpha_modes[0],
            present_mode: surface_caps.present_modes[0],
            height: size.height,
            width: size.width,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::describe()
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    entry_point: "fs_main",
                    module: &shader,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    polygon_mode: wgpu::PolygonMode::Fill,
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    conservative: false,
                    unclipped_depth: false,
                    strip_index_format: None,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });


        let v_buff = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let i_buff = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX
            }
        );

        Self {
            window,
            config,
            device,
            queue,
            size,
            surface,
            pipeline,
            v_buff, 
            i_buff
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.height <= 0 || size.width <= 0 {
            return;
        }

        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

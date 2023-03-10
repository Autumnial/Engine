use std::vec;

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Backends};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

struct Batch {
    v_buff: Option<wgpu::Buffer>,
    i_buff: Option<wgpu::Buffer>,
    vertices: Vec<Vertex>,
    items: u32,
}

impl Batch {
    fn new() -> Self {
        Self {
            v_buff: None,
            i_buff: None,
            vertices: Vec::new(),
            items: 0,
        }
    }

    fn add_square(&mut self, square: Square, device: &wgpu::Device) {
        #[rustfmt::skip]
        self.vertices.push(Vertex {
            position: [
                square.position[0],
                square.position[1],
                0.0
            ],
            colour: square.colour,
        });
        self.vertices.push(Vertex {
            position: [
                square.position[0],
                square.position[1] - square.size,
                0.0,
            ],
            colour: square.colour,
        });
        self.vertices.push(Vertex {
            position: [
                square.position[0] + square.size,
                square.position[1],
                0.0,
            ],
            colour: square.colour,
        });
        self.vertices.push(Vertex {
            position: [
                square.position[0] + square.size,
                square.position[1] - square.size,
                0.0,
            ],
            colour: square.colour,
        });

        self.items += 1;

        self.calculate_buffers(device);
    }

    fn calculate_buffers(&mut self, device: &wgpu::Device) {
        let mut indices: Vec<u16> = Vec::new();

        for i in 0..self.items as u16 {
            let offset = 1 * i;
            indices.push(0 + offset);
            indices.push(1 + offset);
            indices.push(2 + offset);
            indices.push(3 + offset);
            indices.push(2 + offset);
            indices.push(1 + offset);
        }

        self.v_buff = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        self.i_buff = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        ));
    }
}

struct Renderer {
    batches: Vec<Batch>,
    max_items_in_batch: u32,
}

impl Renderer {
    fn new(max_items_in_batch: u32) -> Self {
        let mut batches = Vec::new();
        batches.push(Batch::new());

        Self {
            batches,
            max_items_in_batch,
        }
    }

    fn add_square(&mut self, square: Square, device: &mut wgpu::Device) {
        if self.batches.last().unwrap().items == self.max_items_in_batch {
            self.batches.push(Batch::new());
        }

        self.batches.last_mut().unwrap().add_square(square, device);
    }
}

struct Square {
    position: [f32; 2],
    colour: [f32; 3],
    size: f32,
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

struct Camera {
    target: cgmath::Point3<f32>,
    eye: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    width: f32,
    height: f32,
}

impl Camera {
    fn get_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let half_width = self.width / 2f32;
        let half_height = self.height / 2f32;

        let projection = cgmath::ortho(
            -half_width,
            half_width,
            half_height,
            -half_height,
            -5f32,
            100f32,
        );

        return OPENGL_TO_WGPU_MATRIX * projection * view;
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_projection(&mut self, cam: &Camera) {
        self.view_proj = cam.get_projection_matrix().into();
    }
}
pub struct App {
    state: State,
    renderer: Renderer,
    camera: Camera,
    // entities: Vec<Square>,
}

impl App {
    pub async fn run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Sexy Little Engine")
            .with_inner_size(PhysicalSize::new(800, 800))
            .build(&event_loop)
            .unwrap();

        let mut app = App::new(window).await;

        // for _ in 0..2_000{
        //     app.add_square(Square {
        //         position: [-0.5, 0.5],
        //         colour: [1f32; 3],
        //         size: 1f32,
        //     });
        // }

        app.add_square(Square {
            position: [0.0, 0.0],
            colour: [1.0, 0.0, 0.0],
            size: 3.0,
        });

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
        let mut state = State::new(window).await;

        let renderer = Renderer::new(1000);

        let camera = Camera {
            eye: (-2.0, 2.0, -10.0).into(),
            target: (-2.0, 2.0, 0.0).into(),
            height: 8f32,
            width: 8f32,
            up: cgmath::Vector3::unit_y(),
        };

        state.camera_uniform.update_projection(&camera);
        state.queue.write_buffer(
            &state.camera_buffer,
            0,
            bytemuck::cast_slice(&[state.camera_uniform]),
        );
        // let entities = Vec::new();

        Self {
            state,
            renderer,
            camera,
            // entities,
        }
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
            render_pass.set_bind_group(0, &self.state.camera_bind_group, &[]);

            for batch in &self.renderer.batches {
                let v_buff = batch.v_buff.as_ref().unwrap();
                let i_buff = batch.i_buff.as_ref().unwrap();

                render_pass.set_vertex_buffer(0, v_buff.slice(..));

                render_pass.set_index_buffer(
                    i_buff.slice(..),
                    wgpu::IndexFormat::Uint16,
                );

                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn add_square(&mut self, square: Square) {
        self.renderer.add_square(square, &mut self.state.device);
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
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
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

        let camera_uniform = CameraUniform::new();

        let camera_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                contents: bytemuck::cast_slice(&[camera_uniform]),
                label: Some("camera buffer"),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        // should be last in this method, so we can add  bind groups and all that jazz if we wanna

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::describe()],
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

        Self {
            window,
            config,
            device,
            queue,
            size,
            surface,
            pipeline,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
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

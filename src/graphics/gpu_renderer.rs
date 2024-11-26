use async_std::task;
use sdl2::video::Window;
use wgpu::{util::DeviceExt, SurfaceTargetUnsafe, SurfaceConfiguration};
use crate::types::{light::Light, pixel::Pixel, uniforms::Uniforms, view_state::ViewState};

pub struct GpuRenderer<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    surface_config: SurfaceConfiguration,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    pixels: Vec<Pixel>,
    canvas_width: f32,
    canvas_height: f32,
    batch_size: usize,
}

impl GpuRenderer<'_> {
    pub async fn new(window: &Window) -> GpuRenderer<'static> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(window).unwrap()).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = request_device(&adapter).await;
        let batch_size = device.limits().max_compute_workgroups_per_dimension as usize;

        let (canvas_width, canvas_height) = window.drawable_size();
        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: canvas_width,
            height: canvas_height,
            present_mode: wgpu::PresentMode::FifoRelaxed,
            desired_maximum_frame_latency: 3,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let shader_source = include_str!("shader.wgsl");
        let shader_module = create_shader_module(&device, shader_source);

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: None,
            module: &shader_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        GpuRenderer {
            device,
            queue,
            surface,
            surface_config,
            compute_pipeline,
            render_pipeline,
            pixels: Vec::new(),
            canvas_width: canvas_width as f32,
            canvas_height: canvas_height as f32,
            batch_size
        }
    }
}

impl GpuRenderer<'_> {
    pub fn render(&mut self, view_state: &ViewState, light: &Light) {
        let uniforms = Uniforms {
            angle_x: view_state.angle_x,
            angle_y: view_state.angle_y,
            angle_z: view_state.angle_z,
            c_angle_x: view_state.c_angle_x,
            c_angle_y: view_state.c_angle_y,
            c_angle_z: view_state.c_angle_z,
            scale: view_state.scale,
            perspective_scale: view_state.perspective_scale,
            canvas_width: self.canvas_width,
            canvas_height: self.canvas_height,
            light_x: light.x,
            light_y: light.y,
            light_z: light.z,
            intensity: light.intensity,
            camera_x: view_state.camera_x,
            camera_y: view_state.camera_y,
            camera_z: view_state.camera_z,
            ref_x: view_state.ref_x,
            ref_y: view_state.ref_y,
            ref_z: view_state.ref_z,
        };

        let buffer_size = (self.canvas_width * self.canvas_height) as usize;
        let uniform_buffer = create_uniform_buffer(&self.device, uniforms);
        let depth_buffer = create_depth_buffer(&self.device, buffer_size);
        let depth_check_buffer = create_depth_check_buffer(&self.device, buffer_size);
        let lock_buffer = create_lock_buffer(&self.device, buffer_size);

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let num_batches = (self.pixels.len() + self.batch_size - 1) / self.batch_size;
        for batch_index in 0..num_batches {
            let start = batch_index * self.batch_size;
            let end = std::cmp::min(start + self.batch_size, self.pixels.len());
            let pixel_batch = &self.pixels[start..end];
            let pixel_buffer =
                create_pixel_buffer(&self.device, pixel_batch.len() * std::mem::size_of::<Pixel>());

            let bind_group_layout = self.compute_pipeline.get_bind_group_layout(0);
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 1, resource: pixel_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&view), },
                    wgpu::BindGroupEntry { binding: 3, resource: depth_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 4, resource: depth_check_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 5, resource: lock_buffer.as_entire_binding(), },
                ],
                label: None,
            });

            self.queue.write_buffer(&pixel_buffer, 0, bytemuck::cast_slice(&pixel_batch));

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Main Compute Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.compute_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups((end - start) as u32, 1, 1);
            }

            self.queue.submit(Some(encoder.finish()));
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..1, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        
        task::block_on(async { self.device.poll(wgpu::Maintain::Wait) });
    }

    pub fn load_pixels(&mut self, new_pixels: Vec<Pixel>) {
        self.pixels.extend(new_pixels);
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas_width = width as f32;
        self.canvas_height = height as f32;
        self.surface_config.width = width as u32;
        self.surface_config.height = height as u32;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

fn create_uniform_buffer(device: &wgpu::Device, uniforms: Uniforms) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn create_pixel_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Pixel Buffer"),
        size: size as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_depth_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Depth Buffer"),
        size: (std::mem::size_of::<f32>() * size) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

fn create_depth_check_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Depth Check Buffer"),
        size: (std::mem::size_of::<u32>() * size) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

fn create_lock_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Lock Buffer"),
        size: (std::mem::size_of::<u32>() * size) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

async fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::BGRA8UNORM_STORAGE,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .unwrap()
}

fn create_shader_module(device: &wgpu::Device, source: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

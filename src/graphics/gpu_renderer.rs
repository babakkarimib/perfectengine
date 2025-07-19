use async_std::task;
use sdl2::{pixels::PixelFormatEnum, render::TextureCreator, video::WindowContext};
use wgpu::{util::DeviceExt, Buffer};
use crate::types::{light::Light, pixel::Pixel, renderer::Renderer, uniforms::Uniforms, view_state::ViewState};

pub struct GpuRenderer<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    texture: sdl2::render::Texture<'a>,
    raytracing_compute_pipeline: wgpu::ComputePipeline,
    lighting_compute_pipeline: wgpu::ComputePipeline,
    projection_compute_pipeline: wgpu::ComputePipeline,
    pixels: Vec<Pixel>,
    canvas_width: f32,
    canvas_height: f32,
    batch_size: usize,
    depth_map_buffer: Option<Buffer>,
}

impl GpuRenderer<'_> {
    pub async fn new<'a>(
        canvas: sdl2::render::Canvas<sdl2::video::Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> GpuRenderer<'a> {
        let (canvas_width, canvas_height) = canvas.output_size().unwrap();
        let texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA8888, canvas_width, canvas_height)
            .unwrap();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                ..wgpu::RequestAdapterOptions::default()
            })
            .await
            .unwrap();

        let (device, queue) = request_device(&adapter).await;
        let batch_size = device.limits().max_compute_workgroups_per_dimension as usize;

        let shader_module = create_shader_module("Raytracing Compute Shader", &device, include_str!("raytracing_shader.wgsl"));
        let raytracing_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raytracing Compute Pipeline"),
            layout: None,
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let shader_module = create_shader_module("Lighting Compute Shader", &device, include_str!("lighting_shader.wgsl"));
        let lighting_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Lighting Compute Pipeline"),
            layout: None,
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let shader_module = create_shader_module("Projection Compute Shader", &device, include_str!("projection_shader.wgsl"));
        let projection_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Projection Compute Pipeline"),
            layout: None,
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        GpuRenderer {
            device,
            queue,
            canvas,
            texture_creator,
            texture,
            raytracing_compute_pipeline,
            lighting_compute_pipeline,
            projection_compute_pipeline,
            pixels: Vec::new(),
            canvas_width: canvas_width as f32,
            canvas_height: canvas_height as f32,
            batch_size,
            depth_map_buffer: None
        }
    }
}

impl Renderer<'_> for GpuRenderer<'_> {
    fn render(&mut self, view_state: &ViewState, light: &Light) {
        let buffer_size = (self.canvas_width * self.canvas_height) as usize;
        let raytracing_depth_buffer = create_depth_buffer(&self.device, buffer_size);
        let raytracing_depth_map_buffer = create_depth_map_buffer(&self.device, buffer_size);
        let projection_depth_buffer = create_depth_buffer(&self.device, buffer_size);
        let projection_depth_map_buffer = create_depth_map_buffer(&self.device, buffer_size);
        let img_buffer = create_image_buffer(&self.device, buffer_size);
        let staging_buffer = create_staging_buffer(&self.device, buffer_size);
        let lock_buffer = create_lock_buffer(&self.device, buffer_size);

        let uniforms = Uniforms {
            angle_x: view_state.angle_x,
            angle_y: view_state.angle_y,
            angle_z: view_state.angle_z,
            c_angle_x: view_state.c_angle_x,
            c_angle_y: view_state.c_angle_y,
            c_angle_z: view_state.c_angle_z,
            l_angle_x: view_state.l_angle_x,
            l_angle_y: view_state.l_angle_y,
            l_angle_z: view_state.l_angle_z,
            scale: view_state.scale,
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
            z_offset: view_state.z_offset,
        };
        let uniform_buffer = create_uniform_buffer(&self.device, uniforms);

        // TODO: iterate over objects here and for each render the pixels

        let num_batches = (self.pixels.len() + self.batch_size - 1) / self.batch_size;
        for batch_index in 0..num_batches {
            let start = batch_index * self.batch_size;
            let end = std::cmp::min(start + self.batch_size, self.pixels.len());
            let pixel_batch = &self.pixels[start..end];
            let pixel_buffer =
                create_pixel_buffer(&self.device, pixel_batch.len() * std::mem::size_of::<Pixel>());

            self.queue.write_buffer(&pixel_buffer, 0, bytemuck::cast_slice(&pixel_batch));

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Raytracing Encoder"),
            });

            let bind_group_layout = self.raytracing_compute_pipeline.get_bind_group_layout(0);
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 1, resource: pixel_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 2, resource: raytracing_depth_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 3, resource: raytracing_depth_map_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 4, resource: lock_buffer.as_entire_binding(), },
                ],
                label: None,
            });

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Raytracing Compute Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.raytracing_compute_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups((end - start) as u32, 1, 1);
            }

            self.queue.submit(Some(encoder.finish()));
        }

        for batch_index in 0..num_batches {
            let start = batch_index * self.batch_size;
            let end = std::cmp::min(start + self.batch_size, self.pixels.len());
            let pixel_batch = &self.pixels[start..end];
            let pixel_buffer =
                create_pixel_buffer(&self.device, pixel_batch.len() * std::mem::size_of::<Pixel>());

            self.queue.write_buffer(&pixel_buffer, 0, bytemuck::cast_slice(&pixel_batch));
            
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

            let bind_group_layout = self.lighting_compute_pipeline.get_bind_group_layout(0);
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 1, resource: pixel_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 2, resource: raytracing_depth_buffer.as_entire_binding(), },
                ],
                label: None,
            });

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Lighting Compute Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.lighting_compute_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups((end - start) as u32, 1, 1);
            }

            let bind_group_layout = self.projection_compute_pipeline.get_bind_group_layout(0);
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 1, resource: pixel_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 2, resource: img_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 3, resource: projection_depth_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 4, resource: projection_depth_map_buffer.as_entire_binding(), },
                    wgpu::BindGroupEntry { binding: 5, resource: lock_buffer.as_entire_binding(), },
                ],
                label: None,
            });

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Projection Compute Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.projection_compute_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups((end - start) as u32, 1, 1);
            }

            self.queue.submit(Some(encoder.finish()));
        }

        self.depth_map_buffer = Some(projection_depth_map_buffer);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Image Encoder"),
        });

        encoder.copy_buffer_to_buffer(&img_buffer, 0, &staging_buffer, 0, (std::mem::size_of::<u32>() * buffer_size) as u64);
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        let _ = self.device.poll(wgpu::PollType::wait());
        let pixel_data= task::block_on(async { 
            if let Ok(Ok(())) = receiver.recv_async().await {
                let data = buffer_slice.get_mapped_range();
                let pixel_data: Vec<u8> = bytemuck::cast_slice(&data).to_vec();

                drop(data);
                staging_buffer.unmap();

                pixel_data
            } else {
                panic!("failed to run compute on gpu!")
            }
        });

        self.texture.update(None, &pixel_data, self.canvas_width as usize * 4).unwrap();
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }

    fn load_pixels(&mut self, new_pixels: Vec<Pixel>) {
        self.pixels.extend(new_pixels);
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.canvas_width = width as f32;
        self.canvas_height = height as f32;
        self.texture = self.texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, width, height)
        .unwrap();
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

fn create_depth_map_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Depth Map Buffer"),
        size: (std::mem::size_of::<u32>() * size) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

fn create_image_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Image Buffer"),
        size: (std::mem::size_of::<u32>() * size) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

fn create_staging_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Image Buffer"),
        size: (std::mem::size_of::<u32>() * size) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
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
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits {
                    max_texture_dimension_1d: 4096,
                    max_texture_dimension_2d: 4096,
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            },
        )
        .await
        .unwrap()
}

fn create_shader_module(label: &str, device: &wgpu::Device, source: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

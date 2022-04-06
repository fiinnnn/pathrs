mod compute_pipeline;
mod render_pipeline;

use crate::camera::Camera;
use crate::renderer::compute_pipeline::ComputePipeline;
use crate::renderer::render_pipeline::RenderPipeline;

pub struct Renderer {
    width: u32,
    height: u32,
    viewport_buffer: wgpu::Buffer,
    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    render_target: wgpu::TextureView,
    imgui_renderer: imgui_wgpu::Renderer,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        imgui_ctx: &mut imgui::Context,
        surface_config: &wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
    ) -> Self {
        let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Viewport buffer"),
            size: std::mem::size_of::<pathrs_shared::Viewport>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let compute_pipeline = ComputePipeline::new(device);
        let render_pipeline = RenderPipeline::new(device, surface_config);

        let render_target_texture = create_render_target_texture(device, width, height);
        let render_target =
            render_target_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let imgui_renderer = imgui_wgpu::Renderer::new(
            imgui_ctx,
            device,
            queue,
            imgui_wgpu::RendererConfig {
                texture_format: surface_config.format,
                ..Default::default()
            },
        );

        Self {
            width,
            height,
            viewport_buffer,
            compute_pipeline,
            render_pipeline,
            render_target,
            imgui_renderer,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let render_target_texture = create_render_target_texture(device, width, height);
        self.render_target =
            render_target_texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.width = width;
        self.height = height;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        texture: &wgpu::SurfaceTexture,
        imgui_frame: imgui::Ui,
        queue: &wgpu::Queue,
        camera: &Camera,
    ) {
        puffin::profile_function!();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        queue.write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::cast_slice(&[camera.as_viewport()]),
        );

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &self.compute_pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.render_target),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.viewport_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render bind group"),
            layout: &self.render_pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.render_target),
            }],
        });

        {
            puffin::profile_scope!("compute pass");

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute pass"),
            });
            compute_pass.set_pipeline(&self.compute_pipeline.pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            compute_pass.dispatch(self.width, self.height, 1);
        }

        {
            puffin::profile_scope!("render pass");

            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline.pipeline);
            render_pass.set_bind_group(0, &render_bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            match self
                .imgui_renderer
                .render(imgui_frame.render(), queue, device, &mut render_pass)
            {
                Err(e) => log::error!("imgui render failed: {}", e),
                _ => (),
            };
        }

        queue.submit(Some(encoder.finish()));
    }
}

fn create_render_target_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Render target texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
    })
}

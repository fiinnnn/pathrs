pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        // let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        //     label: Some("Compute shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/compute.wgsl").into()),
        // });

        let shader_source = wgpu::util::make_spirv_raw(include_bytes!(env!("pathrs_shader.spv")));
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Compute shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Owned(shader_source.into_owned())),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute bind group layout"),
            entries: &[
                // render target
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: wgpu::TextureFormat::Rgba32Float,
                    },
                    count: None,
                },
                // camera
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main_cs",
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

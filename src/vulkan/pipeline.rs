use super::*;

pub struct VulkanPipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub vert_shader: vk::ShaderModule,
    pub frag_shader: vk::ShaderModule,
}

impl VulkanPipeline {
    pub fn create(
        bvk: &BabyVulkan,
        render: &VulkanRender,
        extent: vk::Extent2D,
        bindings: &[vk::VertexInputBindingDescription],
        attributes: &[vk::VertexInputAttributeDescription],
        push_constants: &[vk::PushConstantRange],
    ) -> Option<Self> {
        //  Create the shaders
        let vert_shader = Self::create_shader_module(bvk, "./vertex.spv")?;
        let frag_shader = Self::create_shader_module(bvk, "./fragment.spv")?;

        //  Create Shader Stage Info
        let entry_point = CString::new("main").ok()?;
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .name(&entry_point)
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader)
            .build();
        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .name(&entry_point)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader)
            .build();

        //  Create Vertex Input State Info
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            //  Our triangles are currently builtin
            .vertex_binding_descriptions(bindings)
            .vertex_attribute_descriptions(attributes)
            .build();

        //  Create Input Assembly Info
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        //  Create Rasterization Info
        let raster_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_bias_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .build();

        //  Create Multisample Info
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        //  Create Color Blend Info
        let color_blend_state = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .build();

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[color_blend_state])
            .build();

        //  Create Viewport State
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }])
            .scissors(&[vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            }])
            .build();

        //  Create Pipeline Layout
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(push_constants)
            .build();
        let pipeline_layout =
            unsafe { bvk.dev.create_pipeline_layout(&pipeline_layout_info, None) }.ok()?;

        //  Create the Graphics Pipeline
        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&[vert_shader_stage_info, frag_shader_stage_info])
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&raster_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&color_blend_info)
            .render_pass(render.render_pass)
            .subpass(0)
            .layout(pipeline_layout)
            .build();
        let pipeline = unsafe {
            bvk.dev.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphics_pipeline_info],
                None,
            )
        }
        .ok()?
        .into_iter()
        .next()?;

        Some(VulkanPipeline {
            pipeline,
            pipeline_layout,
            vert_shader,
            frag_shader,
        })
    }

    fn create_shader_module(bvk: &BabyVulkan, path: &str) -> Option<vk::ShaderModule> {
        let code = std::fs::read(path).expect(&format!("Failed to read {}", path));
        let shader_info = vk::ShaderModuleCreateInfo::builder()
            .code(unsafe {
                std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / (32 / 8))
            })
            .build();
        unsafe { bvk.dev.create_shader_module(&shader_info, None) }.ok()
    }

    pub fn destroy(&self, bvk: &BabyVulkan) {
        unsafe {
            bvk.dev.destroy_shader_module(self.vert_shader, None);
            bvk.dev.destroy_shader_module(self.frag_shader, None);
            bvk.dev.destroy_pipeline(self.pipeline, None);
            bvk.dev.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

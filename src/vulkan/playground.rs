use super::*;

const FRAME_BUFFER_COUNT: usize = 2;

pub struct VulkanPlayground {
    bvk: BabyVulkan,
    swappy: VulkanSwapchain,
    render: VulkanRender,
    uniform: Uniform<FRAME_BUFFER_COUNT>,
    pipeline: VulkanPipeline,

    vbo: Buffer,
    ibo: Buffer,
    texture: Texture,

    cmd_pool: vk::CommandPool,
    etc_fence: vk::Fence,

    frames: Frames<FRAME_BUFFER_COUNT>,

    start: std::time::Instant,
}

impl VulkanPlayground {
    pub fn create(window: &Window, w: u32, h: u32) -> Option<Self> {
        let mut bvk = BabyVulkan::create(window)?;
        let swappy = VulkanSwapchain::create(&bvk, w, h)?;
        let render = VulkanRender::create(&bvk, &swappy)?;

        let cmd_pool = bvk.create_command_pool()?;
        let etc_cmd_buf = bvk.create_primary_command_buffer(cmd_pool)?;
        let etc_fence = bvk.create_fence(false)?;

        let texture = Texture::create("texture.jpg", &mut bvk, etc_fence, etc_cmd_buf)?;
        let uniform = Uniform::<FRAME_BUFFER_COUNT>::create(&bvk, &texture)?;
        let pipeline = VulkanPipeline::create(
            &bvk,
            &render,
            swappy.extent,
            &[Vertex::bindings()],
            &Vertex::attributes(),
            &[PushConstantData::push_constants()],
            &[uniform.descriptor_set_layout],
        )?;

        //  Define Vertex and Index Data
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5, -0.5],
                color: [1.0, 1.0, 1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                color: [0.85, 0.85, 0.85],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                color: [0.7, 0.7, 0.7],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                color: [0.55, 0.55, 0.55],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.5],
                color: [0.4, 0.4, 0.4],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                color: [0.25, 0.25, 0.25],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                color: [0.1, 0.1, 0.1],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                color: [0.0, 0.0, 0.0],
                uv: [1.0, 1.0],
            },
        ];
        //  Because I'm too lazy to define vertices for every side, the texture will look off. :)
        let indices = vec![
            0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 4, 0, 1, 4, 1, 5, 6, 2, 3, 6, 3, 7, 4, 0, 3, 4, 3,
            7, 1, 5, 6, 1, 6, 2,
        ];

        //  Transfer a vbo Staging Buffer to GPU Memory
        let staging_vbo = Buffer::create_with_data(
            &vertices,
            &bvk,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuOnly,
        )?;
        let vbo = Buffer::create(
            staging_vbo.size,
            &bvk,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk_mem::MemoryUsage::GpuOnly,
        )?;
        Buffer::upload_copy_data(&staging_vbo, &vbo, &bvk, etc_fence, etc_cmd_buf)?;
        staging_vbo.destroy(&mut bvk);

        //  Transfer an ibo Staging Buffer to GPU Memory
        let staging_ibo = Buffer::create_with_data(
            &indices,
            &bvk,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuOnly,
        )?;
        let ibo = Buffer::create(
            staging_ibo.size,
            &bvk,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk_mem::MemoryUsage::GpuOnly,
        )?;
        Buffer::upload_copy_data(&staging_ibo, &ibo, &bvk, etc_fence, etc_cmd_buf)?;
        staging_ibo.destroy(&mut bvk);

        Some(VulkanPlayground {
            vbo,
            ibo,
            texture,

            frames: Frames::create(&bvk, cmd_pool)?,
            cmd_pool,
            etc_fence,

            bvk,
            swappy,
            render,
            uniform,
            pipeline,

            start: std::time::Instant::now(),
        })
    }

    pub fn render(&mut self, window: &Window) -> Option<()> {
        let dims = window.inner_size();
        let w = dims.width;
        let h = dims.height;
        let current_frame = self.frames.get_current_frame();
        let current_cmd_buf = self.frames.cmd_bufs[current_frame];
        let current_render_semaphore = self.frames.render_semaphores[current_frame];
        let current_present_semaphore = self.frames.present_semaphores[current_frame];
        let current_frame_fence = self.frames.frame_fences[current_frame];
        let elapsed = self.start.elapsed().as_millis();
        unsafe {
            //  Wait for the GPU to finish munching on our previous work and resize if neccesary
            assert!(self
                .bvk
                .dev
                .wait_for_fences(&[current_frame_fence], true, std::u64::MAX)
                .is_ok());
            assert!(self.bvk.dev.reset_fences(&[current_frame_fence]).is_ok());
            let (swapchain_image_idx, _suboptimal) =
                match self.swappy.swapchain_ext.acquire_next_image(
                    self.swappy.swapchain,
                    std::u64::MAX,
                    current_present_semaphore,
                    vk::Fence::null(),
                ) {
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        self.resize(w, h);
                        return Some(());
                    }
                    Err(_) => None?,
                    Ok(ret) => ret,
                };

            assert!(self
                .bvk
                .dev
                .reset_command_buffer(current_cmd_buf, vk::CommandBufferResetFlags::empty())
                .is_ok());

            let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            //  Record the command buffer
            self.bvk
                .dev
                .begin_command_buffer(current_cmd_buf, &cmd_begin_info)
                .ok()?;
            {
                let color_clear_value = vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.2, 0.3, 0.5, 1.0],
                    },
                };
                let depth_clear_value = vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                };

                let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(self.render.render_pass)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swappy.extent,
                    })
                    .clear_values(&[color_clear_value, depth_clear_value])
                    .framebuffer(*self.render.framebuffers.get(swapchain_image_idx as usize)?)
                    .build();
                self.bvk.dev.cmd_begin_render_pass(
                    current_cmd_buf,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                {
                    self.bvk.dev.cmd_bind_pipeline(
                        current_cmd_buf,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline.pipeline,
                    );
                    let mut push_constant = PushConstantData {
                        mvp: glm::identity(),
                    };

                    let view_mat = glm::identity();
                    let view_mat = glm::translate(&view_mat, &glm::vec3(0.0, 0.0, -2.0));

                    let model_mat = glm::identity();
                    let model_mat = glm::rotate(
                        &model_mat,
                        elapsed as f32 / 8.0 * (glm::pi::<f32>() / 180.0),
                        &glm::vec3(1.0, 0.0, 1.0),
                    );

                    let perspective = glm::perspective(
                        800.0 / 600.0,
                        90.0 * (glm::pi::<f32>() / 180.0),
                        0.1,
                        100.0,
                    );

                    push_constant.mvp = perspective * view_mat * model_mat;

                    self.bvk.dev.cmd_push_constants(
                        current_cmd_buf,
                        self.pipeline.pipeline_layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        std::slice::from_raw_parts(
                            (&push_constant as *const PushConstantData) as *const u8,
                            std::mem::size_of::<PushConstantData>(),
                        ),
                    );
                    self.bvk
                        .dev
                        .cmd_bind_vertex_buffers(current_cmd_buf, 0, &[self.vbo.buf], &[0]);
                    self.bvk.dev.cmd_bind_index_buffer(
                        current_cmd_buf,
                        self.ibo.buf,
                        0,
                        vk::IndexType::UINT32,
                    );

                    let uniform_data = UniformData {
                        color: glm::vec4(1.0, 0.0, 0.0, 1.0)
                            * ((elapsed as f32 / 500.0).sin() + 1.2),
                    };
                    self.uniform.uniform_bufs[current_frame].map_copy_data(
                        &self.bvk,
                        &uniform_data as *const UniformData as *const u8,
                        std::mem::size_of::<UniformData>(),
                    );

                    self.bvk.dev.cmd_bind_descriptor_sets(
                        current_cmd_buf,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline.pipeline_layout,
                        0,
                        &[self.uniform.descriptor_sets[current_frame]],
                        &[],
                    );

                    //  self.bvk.dev.cmd_draw(self.cmd_buf, 3, 1, 0, 0);
                    self.bvk
                        .dev
                        .cmd_draw_indexed(current_cmd_buf, 36, 1, 0, 0, 0);
                }
                self.bvk.dev.cmd_end_render_pass(current_cmd_buf);
            }
            self.bvk.dev.end_command_buffer(current_cmd_buf).ok()?;

            //  Ready to render!
            let submit_info = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .wait_semaphores(&[current_present_semaphore])
                .signal_semaphores(&[current_render_semaphore])
                .command_buffers(&[current_cmd_buf])
                .build();

            self.bvk
                .dev
                .queue_submit(self.bvk.graphics_queue, &[submit_info], current_frame_fence)
                .unwrap();

            //  Ready to display!
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&[current_render_semaphore])
                .swapchains(&[self.swappy.swapchain])
                .image_indices(&[swapchain_image_idx])
                .build();

            self.frames.advance();
            match self
                .swappy
                .swapchain_ext
                .queue_present(self.bvk.present_queue, &present_info)
            {
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.resize(w, h)?,
                Err(_) => None?,
                _ => {}
            }
        }
        Some(())
    }

    pub fn resize(&mut self, w: u32, h: u32) -> Option<()> {
        unsafe { self.bvk.dev.device_wait_idle().unwrap() };
        self.pipeline.destroy(&self.bvk);
        self.render.destroy(&self.bvk);
        self.swappy.destroy(&self.bvk);
        self.swappy = VulkanSwapchain::create(&self.bvk, w, h)?;
        self.render = VulkanRender::create(&self.bvk, &self.swappy)?;
        self.pipeline = VulkanPipeline::create(
            &self.bvk,
            &self.render,
            self.swappy.extent,
            &[Vertex::bindings()],
            &Vertex::attributes(),
            &[PushConstantData::push_constants()],
            &[self.uniform.descriptor_set_layout],
        )?;
        Some(())
    }
}

impl Drop for VulkanPlayground {
    fn drop(&mut self) {
        unsafe {
            self.bvk.dev.device_wait_idle().unwrap();
        }
        self.vbo.destroy(&mut self.bvk);
        self.ibo.destroy(&mut self.bvk);
        self.texture.destroy(&self.bvk);
        self.uniform.destroy(&mut self.bvk);
        unsafe {
            self.bvk.dev.destroy_fence(self.etc_fence, None);
            self.bvk.dev.destroy_command_pool(self.cmd_pool, None);
        }
        self.frames.destroy(&self.bvk);
        self.pipeline.destroy(&self.bvk);
        self.render.destroy(&self.bvk);
        self.swappy.destroy(&self.bvk);
        self.bvk.destroy();
    }
}

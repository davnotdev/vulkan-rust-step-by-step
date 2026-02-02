use super::*;

pub struct VulkanRender {
    pub depth_image: Image,
    pub depth_image_view: vk::ImageView,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl VulkanRender {
    pub fn create(bvk: &BabyVulkan, swappy: &VulkanSwapchain) -> Option<Self> {
        //  Create Depth Image
        let depth_image_format = vk::Format::D32_SFLOAT;
        let depth_image = Image::create(
            bvk,
            depth_image_format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::Extent3D {
                width: swappy.extent.width,
                height: swappy.extent.height,
                depth: 1,
            },
        )?;
        let depth_image_view = bvk.create_image_view(
            depth_image.image,
            depth_image_format,
            vk::ImageAspectFlags::DEPTH,
        )?;

        //  Create Color Attachment
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swappy.format)
            //  No MSAA
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            //  No stencil, so we don't care
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            //  We're clearing anyway, so this can be undefined
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();
        let color_attachment_ref = vk::AttachmentReference::builder()
            //  Index into the attachments of the render pass itself
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        //  Create Depth Attachment
        let depth_attachment = vk::AttachmentDescription::builder()
            .format(depth_image_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();
        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        //  Create Render Pass Subpass
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .depth_stencil_attachment(&depth_attachment_ref)
            .build();

        let color_dependency = vk::SubpassDependency::builder()
            //  Finish running `vk::SUBPASS_EXTERNAL` (the subpass of the previous frame) before
            //  running subpass # 0
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            //  Make sure you finish running `src_stage_mask`
            //  before running `dst_stage_mask`
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let depth_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .build();

        //  Create Render Pass
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .dependencies(&[color_dependency, depth_dependency])
            .attachments(&[color_attachment, depth_attachment])
            .subpasses(&[subpass])
            .build();
        let render_pass = unsafe { bvk.dev.create_render_pass(&render_pass_info, None) }.ok()?;

        //  Create Framebuffers
        let framebuffers: Vec<vk::Framebuffer> = swappy
            .swapchain_image_views
            .iter()
            .map(|view| {
                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&[*view, depth_image_view])
                    .width(swappy.extent.width)
                    .height(swappy.extent.height)
                    .layers(1)
                    .build();
                unsafe { bvk.dev.create_framebuffer(&framebuffer_info, None).ok() }
            })
            .collect::<Option<_>>()?;

        Some(VulkanRender {
            depth_image,
            depth_image_view,
            render_pass,
            framebuffers,
        })
    }

    pub fn destroy(&mut self, bvk: &BabyVulkan) {
        unsafe {
            bvk.dev.destroy_image_view(self.depth_image_view, None);
            self.depth_image.destroy(bvk);
            self.framebuffers
                .iter()
                .for_each(|framebuffer| bvk.dev.destroy_framebuffer(*framebuffer, None));
            bvk.dev.destroy_render_pass(self.render_pass, None);
        }
    }
}

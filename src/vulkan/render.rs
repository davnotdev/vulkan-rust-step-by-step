use super::*;

pub struct VulkanRender {
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl VulkanRender {
    pub fn create(bvk: &BabyVulkan, swappy: &VulkanSwapchain) -> Option<Self> {
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

        //  Create Render Pass Subpass
        let color_attachment_ref = vk::AttachmentReference::builder()
            //  Index into the attachments of the render pass itself
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .build();

        //  Create Render Pass
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment])
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
                    .attachments(&[*view])
                    .width(swappy.extent.width)
                    .height(swappy.extent.height)
                    .layers(1)
                    .build();
                unsafe { bvk.dev.create_framebuffer(&framebuffer_info, None).ok() }
            })
            .collect::<Option<_>>()?;

        Some(VulkanRender {
            render_pass,
            framebuffers,
        })
    }

    pub fn destroy(&self, bvk: &BabyVulkan) {
        unsafe {
            self.framebuffers
                .iter()
                .for_each(|framebuffer| bvk.dev.destroy_framebuffer(*framebuffer, None));
            bvk.dev.destroy_render_pass(self.render_pass, None);
        }
    }
}

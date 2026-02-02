use super::*;

pub struct VulkanSwapchain {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub swapchain_ext: extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_image_views: Vec<vk::ImageView>,
}

impl VulkanSwapchain {
    pub fn create(bvk: &BabyVulkan, w: u32, h: u32) -> Option<Self> {
        let (surface_caps, surface_formats, surface_presents) = bvk.get_surface_data()?;

        //  Choose Swapchain Extent
        //  Vulkan throws strange errors here.
        //  Perhaps this is an issue with winit? My driver?
        //  Maybe a result of Xorg's latency? I don't know.
        let extent = surface_caps.current_extent;
        let extent = if extent.width >= surface_caps.min_image_extent.width
            && extent.width <= surface_caps.max_image_extent.width
            && extent.height >= surface_caps.min_image_extent.height
            && extent.height <= surface_caps.max_image_extent.height
        {
            extent
        } else {
            vk::Extent2D {
                width: w.clamp(
                    surface_caps.min_image_extent.width,
                    surface_caps.max_image_extent.width,
                ),
                height: h.clamp(
                    surface_caps.min_image_extent.height,
                    surface_caps.max_image_extent.height,
                ),
            }
        };

        //  Choose Swapchain Format
        let format = surface_formats.into_iter().next()?;

        //  Choose Swapchain Present
        let present = [
            vk::PresentModeKHR::FIFO_RELAXED,
            vk::PresentModeKHR::FIFO,
            vk::PresentModeKHR::MAILBOX,
            vk::PresentModeKHR::IMMEDIATE,
        ]
        .into_iter()
        .find(|mode| surface_presents.iter().any(|p| p == mode))?;

        //  Create the Swapchain
        let swapchain_ext = extensions::khr::Swapchain::new(&bvk.instance, &bvk.dev);
        let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(bvk.surface)
            .image_extent(extent)
            .present_mode(present)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .min_image_count(surface_caps.min_image_count)
            .pre_transform(surface_caps.current_transform)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .build();
        let swapchain = unsafe { swapchain_ext.create_swapchain(&swapchain_info, None) }.ok()?;
        let swapchain_images = unsafe { swapchain_ext.get_swapchain_images(swapchain) }.ok()?;
        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .into_iter()
            .map(|img| bvk.create_image_view(img, format.format, vk::ImageAspectFlags::COLOR))
            .collect::<Option<_>>()?;

        Some(VulkanSwapchain {
            format: format.format,
            extent,
            swapchain,
            swapchain_ext,
            swapchain_image_views,
        })
    }

    pub fn destroy(&self, bvk: &BabyVulkan) {
        unsafe {
            self.swapchain_image_views
                .iter()
                .for_each(|view| bvk.dev.destroy_image_view(*view, None));
            self.swapchain_ext.destroy_swapchain(self.swapchain, None);
        }
    }
}

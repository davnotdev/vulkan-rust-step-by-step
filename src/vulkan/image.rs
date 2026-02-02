use super::*;

pub struct Image {
    pub image: vk::Image,
    pub format: vk::Format,
    pub allocation: vk_mem::Allocation,
}

impl Image {
    pub fn create(
        bvk: &BabyVulkan,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        extent: vk::Extent3D,
    ) -> Option<Self> {
        let image_info = vk::ImageCreateInfo::builder()
            .format(format)
            .usage(usage)
            .extent(extent)
            .image_type(vk::ImageType::TYPE_2D)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .build();

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            // usage: vk_mem::MemoryUsage::GpuOnly,
            required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ..Default::default()
        };

        let (image, allocation) =
            unsafe { bvk.alloc.create_image(&image_info, &alloc_info) }.ok()?;

        Some(Image {
            image,
            format,
            allocation,
        })
    }

    pub fn destroy(&mut self, bvk: &BabyVulkan) {
        unsafe {
            bvk.alloc.destroy_image(self.image, &mut self.allocation);
        }
    }
}

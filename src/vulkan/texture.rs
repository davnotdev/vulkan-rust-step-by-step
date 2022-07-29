use super::*;
use stb_image_rust::*;

pub struct Texture {
    pub image: Image,
    pub image_view: vk::ImageView,
    //  Note that you do not need a sampler per texture.
    //  That's wasteful!
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn create(
        file: &str,
        bvk: &mut BabyVulkan,
        fence: vk::Fence,
        cmd_buf: vk::CommandBuffer,
    ) -> Option<Self> {
        //  Load the Image
        let file = std::fs::read(file).unwrap();
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut comp: i32 = 0;
        let image: *mut u8;
        unsafe {
            image = stbi_load_from_memory(
                file.as_ptr(),
                file.len() as i32,
                &mut x as *mut i32,
                &mut y as *mut i32,
                &mut comp as *mut i32,
                STBI_rgb_alpha,
            );
        }

        //  Copy Image Data -> Staging Buffer
        let image_size = (x * y * 4) as usize;
        let staging_image = Buffer::create(
            image_size,
            bvk,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuOnly,
        )?;
        staging_image.map_copy_data(bvk, image, image_size);

        //  Cleanup the Image
        unsafe {
            stbi_image_free(image);
        }

        //  Start Recording on the Command Buffer
        let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe { bvk.dev.begin_command_buffer(cmd_buf, &cmd_begin_info) }.ok()?;

        //  Create Image and Transfer `UNDEFINED` -> `TRANSFER_DST_OPTIMAL`
        let image_extent = vk::Extent3D {
            width: x as u32,
            height: y as u32,
            depth: 1,
        };
        let image = Image::create(
            bvk,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            image_extent,
        )?;

        let range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let image_transfer_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image.image)
            .subresource_range(range)
            .build();

        unsafe {
            bvk.dev.cmd_pipeline_barrier(
                cmd_buf,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_transfer_barrier],
            );
        }

        //  Copy Over out Staging Buffer!
        let image_copy = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .image_extent(image_extent)
            .build();
        unsafe {
            bvk.dev.cmd_copy_buffer_to_image(
                cmd_buf,
                staging_image.buf,
                image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[image_copy],
            );
        }

        //  Transfer `TRANSFER_DST_OPTIMAL` -> `SHADER_READ_ONLY_OPTIMAL`
        let image_transfer_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(image.image)
            .subresource_range(range)
            .build();
        unsafe {
            bvk.dev.cmd_pipeline_barrier(
                cmd_buf,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_transfer_barrier],
            );
        }

        //  Run Our Commands!
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[cmd_buf])
            .build();

        //  Create a Texture Sampler
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .build();
        let sampler = unsafe { bvk.dev.create_sampler(&sampler_info, None) }.ok()?;

        //  Finally, Create an Image View
        let image_view =
            bvk.create_image_view(image.image, image.format, vk::ImageAspectFlags::COLOR)?;

        //  Cleanup the Mess
        unsafe {
            bvk.dev.end_command_buffer(cmd_buf).ok()?;
            bvk.dev
                .queue_submit(bvk.transfer_queue, &[submit_info], fence)
                .ok()?;
            bvk.dev
                .wait_for_fences(&[fence], true, std::u64::MAX)
                .ok()?;
            bvk.dev.reset_fences(&[fence]).ok()?;
            bvk.dev
                .reset_command_buffer(cmd_buf, vk::CommandBufferResetFlags::empty())
                .ok()?;
        }
        staging_image.destroy(bvk);

        Some(Texture {
            image,
            image_view,
            sampler,
        })
    }

    pub fn destroy(&self, bvk: &BabyVulkan) {
        unsafe {
            bvk.dev.destroy_sampler(self.sampler, None);
            bvk.dev.destroy_image_view(self.image_view, None);
            self.image.destroy(bvk);
        }
    }
}

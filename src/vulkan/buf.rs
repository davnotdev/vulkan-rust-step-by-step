use super::*;

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn bindings() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attributes() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(std::mem::size_of::<[f32; 3]>() as u32 * 0)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(std::mem::size_of::<[f32; 3]>() as u32 * 1)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(std::mem::size_of::<[f32; 3]>() as u32 * 2)
                .build(),
        ]
    }
}

#[derive(Clone, Copy)]
pub struct Buffer {
    pub buf: vk::Buffer,
    pub allocation: vk_mem::Allocation,
    pub size: usize,
}

impl Buffer {
    pub fn null() -> Self {
        Buffer {
            buf: vk::Buffer::null(),
            allocation: std::ptr::null_mut(),
            size: 0,
        }
    }

    pub fn create(
        data_size: usize,
        bvk: &BabyVulkan,
        usage: vk::BufferUsageFlags,
        mem_usage: vk_mem::MemoryUsage,
    ) -> Option<Self> {
        //  Create and Allocate the Buffer
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(data_size as u64)
            .usage(usage)
            .build();
        let alloc_info = vk_mem::AllocationCreateInfo::new().usage(mem_usage);
        let (buf, allocation, _) =
            unsafe { bvk.alloc.create_buffer(&buffer_info, &alloc_info) }.unwrap();

        Some(Buffer {
            buf,
            allocation,
            size: data_size,
        })
    }

    pub fn create_with_data<T>(
        cpu_data: &Vec<T>,
        bvk: &BabyVulkan,
        usage: vk::BufferUsageFlags,
        mem_usage: vk_mem::MemoryUsage,
    ) -> Option<Self> {
        let cpu_data_size = cpu_data.len() * std::mem::size_of::<T>();

        let buf = Self::create(cpu_data_size, bvk, usage, mem_usage)?;
        buf.map_copy_data(bvk, cpu_data.as_ptr() as *const u8, cpu_data_size);

        Some(buf)
    }

    pub fn map_copy_data(&self, bvk: &BabyVulkan, ptr: *const u8, size: usize) -> Option<()> {
        //  Fill the Buffer
        unsafe {
            let data = bvk.alloc.map_memory(self.allocation).ok()?;
            std::ptr::copy_nonoverlapping::<u8>(ptr, data, size);
            bvk.alloc.unmap_memory(self.allocation);
        };
        Some(())
    }

    pub fn upload_copy_data(
        src: &Self,
        dst: &Self,
        bvk: &BabyVulkan,
        fence: vk::Fence,
        cmd_buf: vk::CommandBuffer,
    ) -> Option<()> {
        unsafe {
            let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();
            bvk.dev
                .begin_command_buffer(cmd_buf, &cmd_begin_info)
                .ok()?;

            assert!(src.size == dst.size);
            let copy_info = vk::BufferCopy::builder().size(src.size as u64).build();
            bvk.dev
                .cmd_copy_buffer(cmd_buf, src.buf, dst.buf, &[copy_info]);

            bvk.dev.end_command_buffer(cmd_buf).ok()?;

            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&[cmd_buf])
                .build();
            bvk.dev
                .queue_submit(bvk.transfer_queue, &[submit_info], fence)
                .ok()?;
            bvk.dev
                .wait_for_fences(&[fence], true, std::u64::MAX)
                .ok()?;

            //  Cleanup
            bvk.dev.reset_fences(&[fence]).ok()?;
            bvk.dev
                .reset_command_buffer(cmd_buf, vk::CommandBufferResetFlags::empty())
                .ok()?;
        }
        Some(())
    }

    pub fn destroy(&self, bvk: &mut BabyVulkan) {
        unsafe {
            bvk.alloc.destroy_buffer(self.buf, self.allocation);
        }
    }
}

#[repr(C)]
pub struct PushConstantData {
    pub mvp: glm::Mat4,
}

impl PushConstantData {
    pub fn push_constants() -> vk::PushConstantRange {
        vk::PushConstantRange::builder()
            .offset(0)
            .size(std::mem::size_of::<Self>() as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()
    }
}

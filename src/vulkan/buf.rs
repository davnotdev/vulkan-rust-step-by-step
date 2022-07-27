use super::*;

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn bindings() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attributes() -> [vk::VertexInputAttributeDescription; 2] {
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
        ]
    }
}

pub struct Buffer {
    pub buf: vk::Buffer,
    pub allocation: vk_mem::Allocation,
}

impl Buffer {
    pub fn create<T>(cpu_data: &Vec<T>, bvk: &BabyVulkan, usage: vk::BufferUsageFlags) -> Option<Self> {
        let cpu_data_size = cpu_data.len() * std::mem::size_of::<T>();

        //  Create and Allocate the Buffer
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(cpu_data_size as u64)
            .usage(usage)
            .build();
        let alloc_info = vk_mem::AllocationCreateInfo::new().usage(vk_mem::MemoryUsage::CpuToGpu);
        let (buf, allocation, _) =
            unsafe { bvk.alloc.create_buffer(&buffer_info, &alloc_info) }.unwrap();

        //  Fill the Buffer
        unsafe {
            let data = bvk.alloc.map_memory(allocation).ok()?;

            std::ptr::copy_nonoverlapping::<u8>(
                cpu_data.as_ptr() as *const u8,
                data,
                cpu_data_size,
            );

            bvk.alloc.unmap_memory(allocation);
        }

        Some(Buffer { buf, allocation })
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


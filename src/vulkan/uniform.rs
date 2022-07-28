use super::*;

#[repr(C)]
pub struct UniformData {
    pub color: glm::Vec4,
}

impl UniformData {
    pub fn binding() -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .immutable_samplers(&[])
            //  Note, putting `descriptor_count` in front may cause it to get overritten. Maybe
            //  this is a bug with ash, or it's intentional.
            .descriptor_count(1)
            .build()
    }
}

pub struct Uniform<const N: usize> {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub uniform_bufs: [Buffer; N],
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: [vk::DescriptorSet; N],
}

impl<const N: usize> Uniform<N> {
    pub fn create(bvk: &BabyVulkan) -> Option<Self> {
        //  Create Descriptor Set Layout
        let descriptor_set_layout = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[UniformData::binding()])
            .build();

        let descriptor_set_layout = unsafe {
            bvk.dev
                .create_descriptor_set_layout(&descriptor_set_layout, None)
        }
        .ok()?;

        //  Create Uniform Buffers
        let mut uniform_bufs = [Buffer::null(); N];
        for i in 0..N {
            uniform_bufs[i] = Buffer::create(
                align_uniform_buffer_size(&bvk, std::mem::size_of::<UniformData>()),
                bvk,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
            )?;
        }

        //  Create Descriptor Pool
        let descriptor_pool_size = vk::DescriptorPoolSize::builder()
            .descriptor_count(N as u32)
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .build();

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&[descriptor_pool_size])
            .max_sets(N as u32)
            .build();

        let descriptor_pool =
            unsafe { bvk.dev.create_descriptor_pool(&descriptor_pool_info, None) }.ok()?;

        //  Create Descriptor Sets
        let descriptor_sets_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[descriptor_set_layout; N])
            .descriptor_pool(descriptor_pool)
            .build();

        let descriptor_sets: [vk::DescriptorSet; N] =
            unsafe { bvk.dev.allocate_descriptor_sets(&descriptor_sets_info) }
                .ok()?
                .into_iter()
                .collect::<Vec<vk::DescriptorSet>>()
                .try_into()
                .ok()?;

        //  Configure Descriptor Sets
        descriptor_sets
            .iter()
            .zip(uniform_bufs.iter())
            .for_each(|(&set, uniform_buf)| {
                let buffer_info = vk::DescriptorBufferInfo::builder()
                    .buffer(uniform_buf.buf)
                    .range(std::mem::size_of::<UniformData>() as u64)
                    .offset(0)
                    .build();
                let descriptor_write = vk::WriteDescriptorSet::builder()
                    .dst_set(set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&[buffer_info])
                    .build();
                unsafe { bvk.dev.update_descriptor_sets(&[descriptor_write], &[]) }
            });

        Some(Uniform {
            descriptor_set_layout,
            uniform_bufs,
            descriptor_pool,
            descriptor_sets,
        })
    }

    pub fn destroy(&self, bvk: &mut BabyVulkan) {
        unsafe {
            self.uniform_bufs.iter().for_each(|buf| buf.destroy(bvk));
            bvk.dev
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            bvk.dev.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

//  Each GPU has a unique memory alignment requirement
fn align_uniform_buffer_size(bvk: &BabyVulkan, size: usize) -> usize {
    let props = unsafe { bvk.instance.get_physical_device_properties(bvk.gpu) };
    let min_ubo_alignment = props.limits.min_uniform_buffer_offset_alignment as usize;
    if size > 0 {
        (size + min_ubo_alignment - 1) & !(min_ubo_alignment - 1)
    } else {
        size
    }
}

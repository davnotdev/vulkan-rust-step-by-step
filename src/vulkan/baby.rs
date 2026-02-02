use super::*;

//  First baby steps towards rendering.
pub struct BabyVulkan {
    pub _entry: Entry,
    pub instance: Instance,
    pub surface_ext: extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
    pub queue_families: QueueFamilies,
    pub gpu: vk::PhysicalDevice,
    pub dev: Device,
    pub debug_ext: extensions::ext::DebugUtils,
    pub debug: vk::DebugUtilsMessengerEXT,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub alloc: vk_mem::Allocator,
}

impl BabyVulkan {
    pub fn create(window: &Window) -> Option<Self> {
        let entry = Entry::linked();
        let layers_owned = [CString::new("VK_LAYER_KHRONOS_validation").ok()?];
        let layers: Vec<*const i8> = layers_owned.iter().map(|s| s.as_ptr()).collect();

        //  Create Instance
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CString::new("Hello World").ok()?.as_c_str())
            .api_version(vk::API_VERSION_1_2)
            .build();
        let extensions_owned = [
            extensions::ext::DebugUtils::name(),
            extensions::khr::Surface::name(),
            extensions::khr::XlibSurface::name(),
        ];
        let extensions: Vec<*const i8> = extensions_owned.iter().map(|s| s.as_ptr()).collect();
        let inst_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .push_next(&mut get_dbg_info())
            .build();
        let instance = unsafe { entry.create_instance(&inst_info, None) }.ok()?;

        //  Create Debug Stuff
        let debug_ext = extensions::ext::DebugUtils::new(&entry, &instance);
        let debug =
            unsafe { debug_ext.create_debug_utils_messenger(&get_dbg_info(), None) }.ok()?;

        //  Create Surface
        let dpy = window.xlib_display()?;
        let wnd = window.xlib_window()?;
        let xlib_create_info = vk::XlibSurfaceCreateInfoKHR::builder()
            .dpy(dpy as *mut *const c_void)
            .window(wnd)
            .build();
        let xlib_surface = extensions::khr::XlibSurface::new(&entry, &instance);
        let surface = unsafe { xlib_surface.create_xlib_surface(&xlib_create_info, None) }.ok()?;
        let surface_ext = extensions::khr::Surface::new(&entry, &instance);

        //  Select Physical Device and Queue Families
        let gpus = unsafe { instance.enumerate_physical_devices() }.ok()?;
        let suitable_gpus: Vec<(vk::PhysicalDevice, QueueFamilies)> = gpus
            .into_iter()
            .filter_map(|gpu| {
                //  A suitable gpu must
                //      1: Support graphics + present + tranfer
                QueueFamilies::create(&instance, gpu, surface, &surface_ext)
                    .map(|queue_families| (gpu, queue_families))
            })
            .collect();
        let (gpu, queue_families) = suitable_gpus.into_iter().next()?;

        //  Create Device
        let queue_infos = [
            vk::DeviceQueueCreateInfo::builder()
                .queue_priorities(&[1.0])
                .queue_family_index(queue_families.graphics)
                .build(),
            //  On my current system, all of these queues are the same.
            //  Multiple identical queue families are not allowed here.
            //  vk::DeviceQueueCreateInfo::builder()
            //      .queue_priorities(&[1.0])
            //      .queue_family_index(queue_families.present)
            //      .build(),
            //  vk::DeviceQueueCreateInfo::builder()
            //      .queue_priorities(&[1.0])
            //      .queue_family_index(queue_families.transfer)
            //      .build(),
        ];
        let features = vk::PhysicalDeviceFeatures::builder().build();
        let extensions: Vec<*const i8> = [extensions::khr::Swapchain::name()]
            .into_iter()
            .map(|s| s.as_ptr())
            .collect();
        let dev_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extensions)
            // .enabled_layer_names(&layers)
            .enabled_features(&features)
            .queue_create_infos(&queue_infos)
            .build();
        let dev = unsafe { instance.create_device(gpu, &dev_info, None) }.ok()?;

        //  Get the queues
        let present_queue = unsafe { dev.get_device_queue(queue_families.present, 0) };
        let graphics_queue = unsafe { dev.get_device_queue(queue_families.graphics, 0) };
        let transfer_queue = unsafe { dev.get_device_queue(queue_families.transfer, 0) };

        //  Create the Allocator
        let alloc =
            vk_mem::Allocator::new(vk_mem::AllocatorCreateInfo::new(&instance, &dev, gpu)).ok()?;

        Some(BabyVulkan {
            instance,
            _entry: entry,
            surface,
            surface_ext,
            gpu,
            queue_families,
            dev,
            debug_ext,
            debug,
            graphics_queue,
            present_queue,
            transfer_queue,
            alloc,
        })
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.surface_ext.destroy_surface(self.surface, None);
            self.dev.destroy_device(None);
            self.debug_ext
                .destroy_debug_utils_messenger(self.debug, None);
            self.instance.destroy_instance(None);
        }
    }

    pub fn get_surface_data(
        &self,
    ) -> Option<(
        vk::SurfaceCapabilitiesKHR,
        Vec<vk::SurfaceFormatKHR>,
        Vec<vk::PresentModeKHR>,
    )> {
        Some(unsafe {
            (
                self.surface_ext
                    .get_physical_device_surface_capabilities(self.gpu, self.surface)
                    .ok()?,
                self.surface_ext
                    .get_physical_device_surface_formats(self.gpu, self.surface)
                    .ok()?,
                self.surface_ext
                    .get_physical_device_surface_present_modes(self.gpu, self.surface)
                    .ok()?,
            )
        })
    }

    pub fn create_image_view(
        &self,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> Option<vk::ImageView> {
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .format(format)
            .components(
                vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY)
                    .build(),
            )
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .view_type(vk::ImageViewType::TYPE_2D)
            .build();
        unsafe { self.dev.create_image_view(&image_view_info, None) }.ok()
    }

    pub fn create_command_pool(&self) -> Option<vk::CommandPool> {
        let command_pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(self.queue_families.graphics)
            .build();
        unsafe { self.dev.create_command_pool(&command_pool_info, None) }.ok()
    }

    pub fn create_primary_command_buffer(
        &self,
        pool: vk::CommandPool,
    ) -> Option<vk::CommandBuffer> {
        let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();
        unsafe { self.dev.allocate_command_buffers(&command_buffer_info) }
            .ok()?
            .into_iter()
            .next()
    }

    pub fn create_semaphore(&self) -> Option<vk::Semaphore> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder().build();
        unsafe { self.dev.create_semaphore(&semaphore_info, None) }.ok()
    }

    pub fn create_fence(&self, signaled: bool) -> Option<vk::Fence> {
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(if signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            })
            .build();
        unsafe { self.dev.create_fence(&fence_info, None) }.ok()
    }
}

pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
    pub transfer: u32,
}

impl QueueFamilies {
    pub fn create(
        inst: &Instance,
        gpu: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_ext: &extensions::khr::Surface,
    ) -> Option<QueueFamilies> {
        let mut graphics = None;
        let mut present = None;
        let mut transfer = None;
        let queue_family_props = unsafe { inst.get_physical_device_queue_family_properties(gpu) };
        for (idx, prop) in queue_family_props.iter().enumerate() {
            let idx = idx as u32;
            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics.get_or_insert(idx);
            }
            if prop.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                transfer.get_or_insert(idx);
            }
            if unsafe { surface_ext.get_physical_device_surface_support(gpu, idx, surface) }.ok()? {
                present.get_or_insert(idx);
            }
        }
        Some(QueueFamilies {
            graphics: graphics?,
            present: present?,
            transfer: transfer?,
        })
    }
}

fn get_dbg_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .build()
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

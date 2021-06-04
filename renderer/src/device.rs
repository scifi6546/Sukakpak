use super::find_memorytype_index;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk, Entry,
};
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
};
const DO_BACKTRACE: bool = true;
const PANIC: bool = true;
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    if DO_BACKTRACE {
        println!("{:?}", backtrace::Backtrace::new());
    }
    if PANIC {
        panic!();
    }

    vk::FALSE
}
pub struct Device {
    pub device: ash::Device,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub debug_callback: vk::DebugUtilsMessengerEXT,
    pub debug_utils_loader: DebugUtils,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: Swapchain,
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_queue: vk::Queue,
    pub queue_family_index: u32,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    surface: vk::SurfaceKHR,
    surface_loader: Surface,
    //presered to hold onto refrence
    #[allow(dead_code)]
    entry: Entry,
    #[allow(dead_code)]
    priorities: [f32; 1],
    #[allow(dead_code)]
    layer_names: [CString; 1],
    previously_destroyed: bool,
}
impl Device {
    pub fn new(window: &winit::window::Window, fallback_width: u32, fallback_height: u32) -> Self {
        let entry = unsafe { Entry::new() }.unwrap();
        let app_name = CString::new("example game").unwrap();
        let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layer_names_raw: Vec<*const i8> =
            layer_names.iter().map(|name| name.as_ptr()).collect();
        let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        let debug_utils_name = DebugUtils::name();
        extension_names_raw.push(debug_utils_name.as_ptr());
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&app_name)
            .engine_version(0)
            .api_version(vk::make_version(1, 0, 0));
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names_raw);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .expect("failed to create instance");
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(vulkan_debug_callback));
        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        let debug_callback =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap();
        let surface =
            unsafe { ash_window::create_surface(&entry, &instance, window, None) }.unwrap();
        let pdevices =
            unsafe { instance.enumerate_physical_devices() }.expect("failed to get pdevices");
        let surface_loader = Surface::new(&entry, &instance);

        let (physical_device, queue_family_index) = unsafe {
            pdevices.iter().map(|pdevice| {
                instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .filter_map(|(index, ref info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap()
                                && instance
                                    .get_physical_device_features(*pdevice)
                                    .sampler_anisotropy
                                    >= 1;
                        if supports_graphic_and_surface {
                            println!("supports?");
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
                    .next()
            })
        }
        .filter_map(|v| v)
        .next()
        .expect("could not get physical device");
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let queue_family_index = queue_family_index as u32;
        let device_extension_names_raw = [Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures::builder()
            .shader_clip_distance(true)
            .sampler_anisotropy(true);
        let priorities = [1.0];
        let queue_info = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)
            .build()];
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);
        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }
            .expect("failed to create device");

        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()[0]
        };
        println!("{:?}", surface_format);

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: fallback_width,
                height: fallback_height,
            },
            _ => surface_capabilities.current_extent,
        };

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap()
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader = Swapchain::new(&instance, &device);
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);
        let swapchain =
            unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }.unwrap();

        Self {
            queue_family_index,
            device,
            memory_properties,
            physical_device,
            instance,
            debug_callback,
            debug_utils_loader,
            swapchain,
            swapchain_loader,
            surface_format,
            present_queue,
            priorities,
            layer_names,
            entry,
            surface,
            previously_destroyed: false,
            surface_loader,
        }
    }
    pub fn create_buffer(
        &mut self,
        size: u64,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(sharing_mode);
        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_create_info, None)
                .expect("failed to create buffer")
        };
        let buffer_memory_requirements =
            unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(
                find_memorytype_index(
                    &buffer_memory_requirements,
                    &self.memory_properties,
                    memory_properties,
                )
                .expect("failed to find memory type"),
            );
        let buffer_memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate buffer memory")
        };
        unsafe {
            self.device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("failed to bind buffer");
        };
        (buffer, buffer_memory)
    }
    pub fn find_supported_format(
        &self,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for format in formats {
            let properties = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device, *format)
            };
            if tiling == vk::ImageTiling::LINEAR
                && (properties.linear_tiling_features & features) == features
            {
                return *format;
            } else if tiling == vk::ImageTiling::OPTIMAL
                && (properties.optimal_tiling_features & features) == features
            {
                return *format;
            };
        }
        panic!("format not found")
    }

    /// clears resources, warning once called object is in invalid state
    pub fn free(&mut self) {
        assert!(!self.previously_destroyed);
        unsafe {
            self.device.device_wait_idle().expect("failed to wait");
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_callback, None);
            self.instance.destroy_instance(None);
        }
        self.previously_destroyed = true;
    }
}

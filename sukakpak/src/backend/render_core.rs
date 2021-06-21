use super::CommandPool;
use anyhow::Result;
use ash::{
    extensions::ext::DebugUtils,
    extensions::khr::{Surface as AshSurface, Swapchain},
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk,
};
use nalgebra as na;

use std::ffi::{CStr, CString};
const DO_BACKTRACE: bool = false;
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
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
    vk::FALSE
}
pub struct Core {
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    entry: ash::Entry,
    pub instance: ash::Instance,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    debug_callback: vk::DebugUtilsMessengerEXT,
    pub present_queue: vk::Queue,
    pub queue_family_index: u32,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: Swapchain,
    pub surface_format: vk::SurfaceFormatKHR,
    //Swapchain Info
    pre_transform: vk::SurfaceTransformFlagsKHR,
    swapchain_image_count: u32,
    present_mode: vk::PresentModeKHR,
    //end swapchain info
    surface: vk::SurfaceKHR,
    surface_loader: AshSurface,
    debug_utils_loader: DebugUtils,
}
impl Core {
    pub fn new(
        window: &winit::window::Window,
        create_info: &super::BackendCreateInfo,
    ) -> Result<Self> {
        let entry = unsafe { ash::Entry::new() }?;
        let app_name = CString::new(create_info.name.clone())?;
        cfg_if::cfg_if! {
            if #[cfg(feature="no_validation")]{
                let layer_names:[CString;0] = [];
            }else{
                let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            }
        }
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
        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names_raw);
        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
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
        let surface_loader = AshSurface::new(&entry, &instance);

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

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };
        let mut swapchain_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && swapchain_image_count > surface_capabilities.max_image_count
        {
            swapchain_image_count = surface_capabilities.max_image_count;
        }
        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: create_info.default_size.x,
                height: create_info.default_size.y,
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
            .min_image_count(swapchain_image_count)
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
        Ok(Self {
            physical_device,
            swapchain_loader,
            device,
            entry,
            instance,
            memory_properties,
            debug_callback,
            surface_format,
            present_queue,
            swapchain,
            surface_loader,
            surface,
            debug_utils_loader,
            queue_family_index,
            pre_transform,
            swapchain_image_count,
            present_mode,
        })
    }
    pub fn update_swapchain_resolution(&mut self, new_size: na::Vector2<u32>) -> Result<()> {
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(self.swapchain_image_count)
            .image_color_space(self.surface_format.color_space)
            .image_format(self.surface_format.format)
            .image_extent(vk::Extent2D {
                width: new_size.x,
                height: new_size.y,
            })
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(self.pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(self.present_mode)
            .clipped(true)
            .image_array_layers(1);
        self.swapchain = unsafe {
            self.swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
        }
        .unwrap();

        Ok(())
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
            if (tiling == vk::ImageTiling::LINEAR
                && (properties.linear_tiling_features & features) == features)
                || (tiling == vk::ImageTiling::OPTIMAL
                    && (properties.optimal_tiling_features & features) == features)
            {
                return *format;
            };
        }
        panic!("format not found")
    }
    pub fn copy_buffer(
        &mut self,
        command_pool: &mut CommandPool,
        src: &vk::Buffer,
        dst: &vk::Buffer,
        buffer_size: u64,
    ) {
        unsafe {
            let copy_command = command_pool.create_onetime_buffer(self);
            let copy_region = [*vk::BufferCopy::builder()
                .src_offset(0)
                .dst_offset(0)
                .size(buffer_size)];
            copy_command.core.device.cmd_copy_buffer(
                copy_command.command_buffer[0],
                *src,
                *dst,
                &copy_region,
            );
        }
    }
    /// frees resources. Must only be called once
    pub unsafe fn free(&mut self) {
        self.device.device_wait_idle().expect("failed to wait idle");
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
        self.surface_loader.destroy_surface(self.surface, None);
        self.device.destroy_device(None);
        self.debug_utils_loader
            .destroy_debug_utils_messenger(self.debug_callback, None);
        self.instance.destroy_instance(None);
    }
}

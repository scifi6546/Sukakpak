pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk, Entry,
};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

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

    vk::FALSE
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Hello Triangle")
        .with_inner_size(winit::dpi::LogicalSize::new(1000, 1000))
        .build(&event_loop)
        .unwrap();
    // Creating Vulkan context
    //
    let entry = unsafe { Entry::new() }.unwrap();
    let app_name = CString::new("example game").unwrap();
    let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layer_names_raw: Vec<*const i8> = layer_names.iter().map(|name| name.as_ptr()).collect();
    let surface_extensions = ash_window::enumerate_required_extensions(&window).unwrap();
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
    let instance =
        unsafe { entry.create_instance(&create_info, None) }.expect("failed to create instance");
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .pfn_user_callback(Some(vulkan_debug_callback));
    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_callback =
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap();
    let surface = unsafe { ash_window::create_surface(&entry, &instance, &window, None) }.unwrap();
    let pdevices =
        unsafe { instance.enumerate_physical_devices() }.expect("failed to get pdevices");
    let surface_loader = Surface::new(&entry, &instance);

    let (pdevice, queue_family_index) = unsafe {
        pdevices.iter().map(|pdevice| {
            instance
                .get_physical_device_queue_family_properties(*pdevice)
                .iter()
                .enumerate()
                .filter_map(|(index, ref info)| {
                    let supports_graphic_and_surface = info
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS)
                        && surface_loader
                            .get_physical_device_surface_support(*pdevice, index as u32, surface)
                            .unwrap();
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
    let queue_family_index = queue_family_index as u32;
    let device_extension_name = [Swapchain::name().as_ptr()];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let priorities = [1.0];
    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities)
        .build()];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&extension_names_raw)
        .enabled_features(&features);
    let device = unsafe { instance.create_device(pdevice, &device_create_info, None) }
        .expect("failed ot create device");

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => (),
    });
    println!("Hello, world!");
}

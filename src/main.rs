pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

mod graphics;
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    util::*,
    vk, Entry,
};
use graphics::Context;
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    io::Cursor,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
const WINDOW_HEIGHT: u32 = 1000;
const WINDOW_WIDTH: u32 = 1000;

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
    println!("building context");
    {
        let _context = Context::new("Hello Context", &event_loop, 1000, 1000);
    }

    println!("done building context");
    println!("left context");
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
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
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
    let device_extension_names_raw = [Swapchain::name().as_ptr()];
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
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);
    let device = unsafe { instance.create_device(pdevice, &device_create_info, None) }
        .expect("failed to create device");
    let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
    let surface_format = unsafe {
        surface_loader
            .get_physical_device_surface_formats(pdevice, surface)
            .unwrap()[0]
    };
    println!("{:?}", surface_format);

    let surface_capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(pdevice, surface)
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
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
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
            .get_physical_device_surface_present_modes(pdevice, surface)
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
    let swap_chain =
        unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }.unwrap();
    let present_images = unsafe { swapchain_loader.get_swapchain_images(swap_chain) }.unwrap();
    let present_image_views: Vec<vk::ImageView> = present_images
        .iter()
        .map(|&image| {
            let create_image_view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);
            unsafe { device.create_image_view(&create_image_view_info, None) }
                .expect("failed to create image")
        })
        .collect();
    let frag_shader = read_spv(&mut Cursor::new(include_bytes!("shaders/main.frag.spv")))
        .expect("failed to load fragment shader data");
    let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_shader);
    let frag_shader_module = unsafe {
        device
            .create_shader_module(&frag_shader_info, None)
            .expect("Failed to create fragment shader module")
    };
    let vert_shader = read_spv(&mut Cursor::new(include_bytes!("shaders/main.vert.spv")))
        .expect("failed to load vertex shader data");
    let vert_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vert_shader);
    let vert_shader_module = unsafe {
        device
            .create_shader_module(&vert_shader_info, None)
            .expect("failed to create vertex shader_module")
    };
    let layout_create_info = vk::PipelineLayoutCreateInfo::default();
    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&layout_create_info, None)
            .expect("failed to createlayout")
    };
    let shader_entry_name = CString::new("main").unwrap();
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo {
            module: vert_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            module: frag_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];
    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
        vertex_attribute_description_count: 0,
        vertex_binding_description_count: 0,
        ..Default::default()
    };
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: surface_resolution.width as f32,
        height: surface_resolution.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: surface_resolution,
    }];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);
    let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .depth_bias_enable(false)
        .build();
    let multi_sample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .color_write_mask(vk::ColorComponentFlags::all())
        .build()];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&color_blend_attachment_states);

    let color_attachment = [vk::AttachmentDescription::builder()
        .format(surface_format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build()];
    let color_attachment_refs = [vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build()];
    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_refs)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];

    let subpass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .build()];
    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachment)
        .subpasses(&subpasses)
        .dependencies(&subpass_dependencies);
    let renderpass = unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .expect("failed to create renderpass")
    };
    let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_infos)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multi_sample_state_info)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(renderpass)
        .build();
    let graphics_pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_info], None)
            .expect("failed to create pipeline")[0]
    };
    let framebuffers: Vec<vk::Framebuffer> = present_image_views
        .iter()
        .map(|image_view| {
            let attachments = [*image_view];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&attachments)
                .width(surface_resolution.width)
                .height(surface_resolution.height)
                .layers(1);
            unsafe {
                device
                    .create_framebuffer(&create_info, None)
                    .expect("failed to create_framebuffer")
            }
        })
        .collect();
    let command_pool_create_info =
        vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);
    let pool = unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("failed to create command pool")
    };
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(framebuffers.len() as u32)
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY);
    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("failed to allocate command buffer")
    };

    for (command_buffer, framebuffer) in command_buffers.iter().zip(framebuffers.iter()) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
        unsafe {
            device
                .begin_command_buffer(*command_buffer, &command_buffer_begin_info)
                .expect("failed to create command buffer");
            let renderpass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(renderpass)
                .framebuffer(*framebuffer)
                .render_area(vk::Rect2D {
                    extent: surface_resolution,
                    offset: vk::Offset2D { x: 0, y: 0 },
                })
                .clear_values(&[vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.1, 0.1, 0.1, 1.0],
                    },
                }]);
            device.cmd_begin_render_pass(
                *command_buffer,
                &renderpass_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
            device.cmd_draw(*command_buffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(*command_buffer);
            device
                .end_command_buffer(*command_buffer)
                .expect("failed to create command buffer");
        };
    }
    let fences: Vec<vk::Fence> = command_buffers
        .iter()
        .map(|_| {
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("failed to create fence")
            }
        })
        .collect();
    let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
    let image_available_semaphore =
        unsafe { device.create_semaphore(&semaphore_create_info, None) }
            .expect("failed to create semaphore");
    let render_finished_semaphore =
        unsafe { device.create_semaphore(&semaphore_create_info, None) }
            .expect("failed to create semaphore");

    event_loop.run(move |event, _, control_flow| {
        unsafe {
            let (image_index, _) = swapchain_loader
                .acquire_next_image(
                    swap_chain,
                    u64::MAX,
                    image_available_semaphore,
                    vk::Fence::null(),
                )
                .expect("failed to aquire image");
            device
                .wait_for_fences(&[fences[image_index as usize]], true, u64::MAX)
                .expect("failed to wait for fence");
            device
                .reset_fences(&[fences[image_index as usize]])
                .expect("failed to reset fence");

            let signal_semaphores = [render_finished_semaphore];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[image_available_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[command_buffers[image_index as usize]])
                .signal_semaphores(&signal_semaphores)
                .build();
            device
                .queue_submit(present_queue, &[submit_info], fences[image_index as usize])
                .expect("failed to submit queue");
            let wait_semaphores = [swap_chain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&wait_semaphores)
                .image_indices(&image_indices);
            swapchain_loader
                .queue_present(present_queue, &present_info)
                .expect("failed to present queue");
        }
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                unsafe { device.device_wait_idle().expect("failed to wait idle") };
                *control_flow = ControlFlow::Exit
            }
            _ => (),
        }
    });
}

use super::{Core, DepthBuffer, DescriptorDesc};
use ash::version::DeviceV1_0;
use ash::{util::*, vk};
use nalgebra::Vector2;
use std::{ffi::CString, io::Cursor};
mod shaders;
pub use shaders::{alt_shader, push_shader, PushConstantDesc, ShaderDescription, VertexBufferDesc};
pub struct RenderPipeline {
    pub graphics_pipeline: vk::Pipeline,
    pub renderpass: vk::RenderPass,
}
pub struct GraphicsPipeline {
    fragment_shader: vk::ShaderModule,
    vertex_shader: vk::ShaderModule,
    pub pipeline_layout: vk::PipelineLayout,
    //clears render pipeline on draw
    pub clear_pipeline: RenderPipeline,
    // does not clear color bit on draw
    pub load_pipeline: RenderPipeline,
}
#[derive(Clone, Copy, Debug)]
pub enum PipelineType {
    Present,
    OffScreen,
}
impl GraphicsPipeline {
    pub fn new(
        core: &mut Core,
        shader_data: &ShaderDescription,
        descriptor_layouts: &[vk::DescriptorSetLayout],
        screen_dimensions: Vector2<u32>,
        depth_buffer: &DepthBuffer,

        pipeline_type: PipelineType,
    ) -> Self {
        let frag_shader_data = read_spv(&mut Cursor::new(shader_data.fragment_shader_data.clone()))
            .expect("failed to create shader");
        let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_shader_data);
        let fragment_shader = unsafe {
            core.device
                .create_shader_module(&frag_shader_info, None)
                .expect("failed to create shader")
        };

        let vert_shader_data = read_spv(&mut Cursor::new(shader_data.vertex_shader_data.clone()))
            .expect("failed to create shader");
        let vert_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vert_shader_data);
        let vertex_shader = unsafe {
            core.device
                .create_shader_module(&vert_shader_info, None)
                .expect("failed to create shader")
        };
        let shader_entry_name = CString::new("main").unwrap();
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment_shader,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let binding_description = [shader_data.vertex_buffer_desc.binding_description];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_description)
            .vertex_attribute_descriptions(&shader_data.vertex_buffer_desc.attributes);
        let ranges = shader_data
            .push_constants
            .iter()
            .map(|push| push.range)
            .collect::<Vec<_>>();
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_layouts)
            .push_constant_ranges(&ranges);
        let pipeline_layout = unsafe {
            core.device
                .create_pipeline_layout(&layout_create_info, None)
                .expect("failed to createlayout")
        };

        let clear_pipeline = Self::build_render_pipeline(
            core,
            &shader_stage_create_infos,
            *vertex_input_state_info,
            pipeline_layout,
            screen_dimensions.x,
            screen_dimensions.y,
            vk::AttachmentLoadOp::CLEAR,
            depth_buffer,
            vk::ImageLayout::UNDEFINED,
            match pipeline_type {
                PipelineType::Present => vk::ImageLayout::PRESENT_SRC_KHR,
                PipelineType::OffScreen => vk::ImageLayout::GENERAL,
            },
        );
        let load_pipeline = Self::build_render_pipeline(
            core,
            &shader_stage_create_infos,
            *vertex_input_state_info,
            pipeline_layout,
            screen_dimensions.x,
            screen_dimensions.y,
            vk::AttachmentLoadOp::LOAD,
            depth_buffer,
            match pipeline_type {
                PipelineType::Present => vk::ImageLayout::PRESENT_SRC_KHR,
                PipelineType::OffScreen => vk::ImageLayout::GENERAL,
            },
            match pipeline_type {
                PipelineType::Present => vk::ImageLayout::PRESENT_SRC_KHR,
                PipelineType::OffScreen => vk::ImageLayout::GENERAL,
            },
        );
        GraphicsPipeline {
            fragment_shader,
            vertex_shader,
            pipeline_layout,
            clear_pipeline,
            load_pipeline,
        }
    }
    fn build_render_pipeline(
        core: &mut Core,
        shader_stage_create_infos: &[vk::PipelineShaderStageCreateInfo],
        vertex_input_state_info: vk::PipelineVertexInputStateCreateInfo,
        pipeline_layout: vk::PipelineLayout,
        screen_width: u32,
        screen_height: u32,
        load_op: vk::AttachmentLoadOp,
        depth_buffer: &DepthBuffer,
        //initial layout is ignored if load_op is set to clear
        initial_layout: vk::ImageLayout,
        final_layout: vk::ImageLayout,
    ) -> RenderPipeline {
        let renderpass =
            Self::build_renderpass(core, load_op, depth_buffer, initial_layout, final_layout);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: screen_width as f32,
            height: screen_height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: screen_width,
                height: screen_height,
            },
        }];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::FRONT)
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
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);
        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multi_sample_state_info)
            .color_blend_state(&color_blend_state)
            .depth_stencil_state(&depth_stencil)
            .layout(pipeline_layout)
            .render_pass(renderpass)
            .build();
        let graphics_pipeline = unsafe {
            core.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphics_pipeline_info],
                    None,
                )
                .expect("failed to create pipeline")[0]
        };
        RenderPipeline {
            renderpass,
            graphics_pipeline,
        }
    }
    fn build_renderpass(
        core: &mut Core,
        load_op: vk::AttachmentLoadOp,
        depth_buffer: &DepthBuffer,
        initial_layout: vk::ImageLayout,
        final_layout: vk::ImageLayout,
    ) -> vk::RenderPass {
        let (depth_attachment, depth_attachment_ref) = depth_buffer.get_attachment(load_op);
        let color_attachment = [vk::AttachmentDescription::builder()
            .format(core.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(if load_op == vk::AttachmentLoadOp::CLEAR {
                vk::ImageLayout::UNDEFINED
            } else {
                initial_layout
            })
            .final_layout(final_layout)
            .build()];
        let color_attachment_refs = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .build()];
        let attachments = [color_attachment[0], depth_attachment];
        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);
        unsafe {
            core.device
                .create_render_pass(&render_pass_create_info, None)
                .expect("failed to create renderpass")
        }
    }
    pub fn free(&mut self, core: &mut Core) {
        unsafe {
            let free_pipeline = |pipeline: &RenderPipeline| {
                core.device
                    .destroy_pipeline(pipeline.graphics_pipeline, None);
                core.device.destroy_render_pass(pipeline.renderpass, None);
            };
            free_pipeline(&self.clear_pipeline);
            free_pipeline(&self.load_pipeline);
            core.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            core.device
                .destroy_shader_module(self.fragment_shader, None);
            core.device.destroy_shader_module(self.vertex_shader, None);
        }
    }
}

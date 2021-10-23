use anyhow::{bail, Result};
use ass_wgl::Shader as AssShader;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModule {
    pub shader: AssShader,
    fragment_shader: WebGlShader,
    vertex_shader: WebGlShader,
    pub program: WebGlProgram,
}
impl ShaderModule {
    fn make_shader(
        source: &str,
        stage: ShaderStage,
        context: &mut WebGl2RenderingContext,
    ) -> Result<WebGlShader> {
        let shader = context.create_shader(match stage {
            ShaderStage::Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            ShaderStage::Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
        });
        if shader.is_none() {
            bail!("failed to create shader")
        }
        let shader = shader.unwrap();
        context.shader_source(&shader, source);
        context.compile_shader(&shader);
        let success = context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .expect("webgl returned unexpected type");
        if !success {
            bail!("failed to compile shader, {:#?}", success)
        }
        Ok(shader)
    }
    fn make_program(
        fragment_shader: &WebGlShader,
        vertex_shader: &WebGlShader,
        context: &mut WebGl2RenderingContext,
    ) -> Result<WebGlProgram> {
        let program = context
            .create_program()
            .expect("failed to build shader program");
        context.attach_shader(&program, fragment_shader);
        context.attach_shader(&program, vertex_shader);
        context.link_program(&program);
        let success = context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .expect("webgl returned invalid type");
        if success {
            Ok(program)
        } else {
            bail!("failed to link shader program")
        }
    }
    pub fn from_json_str(json: &str, context: &mut WebGl2RenderingContext) -> Result<Self> {
        let shader = AssShader::from_json_str(json)?;
        let fragment_shader =
            Self::make_shader(&shader.fragment_shader, ShaderStage::Fragment, context)?;
        let vertex_shader = Self::make_shader(&shader.vertex_shader, ShaderStage::Vertex, context)?;
        let program = Self::make_program(&fragment_shader, &vertex_shader, context)?;

        Ok(Self {
            fragment_shader,
            vertex_shader,
            program,
            shader,
        })
    }
    pub fn basic_shader(context: &mut WebGl2RenderingContext) -> Result<Self> {
        Self::from_json_str(include_str!("../../shaders/v2/v2_test.ass_glsl"), context)
    }
}

use anyhow::Result;
use ass_lib::ShaderIR;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Write, path::Path};

/// Options for building shader
pub struct Options {
    /// run in verbose mode printing to stdout
    pub verbose: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self { verbose: false }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShaderStage {
    Vertex,
    Fragment,
}
/// GLSL shader module
#[derive(Deserialize, Serialize)]
pub struct Shader {
    pub fragment_shader: String,
    pub vertex_shader: String,
}
impl Shader {
    const EXTENSION: &'static str = "ass_glsl";
    fn write_string(
        ir: &ShaderIR,
        _options: &Options,
        shader_stage: ShaderStage,
    ) -> Result<(String, naga::back::glsl::ReflectionInfo)> {
        let mut buffer = String::new();
        let shader_options = &naga::back::glsl::Options {
            version: naga::back::glsl::Version::Embedded(300),
            writer_flags: naga::back::glsl::WriterFlags::ADJUST_COORDINATE_SPACE,
            binding_map: BTreeMap::new(),
        };
        let pipeline_options = &naga::back::glsl::PipelineOptions {
            entry_point: match shader_stage {
                ShaderStage::Fragment => ass_lib::FRAGMENT_SHADER_MAIN.to_string(),
                ShaderStage::Vertex => ass_lib::VERTEX_SHADER_MAIN.to_string(),
            },
            shader_stage: match shader_stage {
                ShaderStage::Fragment => naga::ShaderStage::Fragment,
                ShaderStage::Vertex => naga::ShaderStage::Vertex,
            },
        };

        let mut writer = naga::back::glsl::Writer::new(
            &mut buffer,
            &ir.module,
            &ir.info,
            shader_options,
            pipeline_options,
        )?;
        let info = writer.write()?;
        Ok((buffer, info))
    }
    pub fn from_ir(ir: ShaderIR, options: Options) -> Result<Self> {
        let (fragment_shader, _frag_info) =
            Self::write_string(&ir, &options, ShaderStage::Fragment)?;
        if options.verbose {
            println!("fragment shader:\n{}", fragment_shader)
        }
        let (vertex_shader, _vert_info) = Self::write_string(&ir, &options, ShaderStage::Vertex)?;

        if options.verbose {
            println!("vertex shader:\n{}", vertex_shader)
        }

        Ok(Self {
            fragment_shader,
            vertex_shader,
        })
    }
    pub fn to_json_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    /// writes shader to disk along with data required for graphics engine
    pub fn write_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json_string = self.to_json_string()?;
        let new_path = path.as_ref().with_extension(Self::EXTENSION);
        let mut file = File::create(new_path)?;
        file.write_all(json_string.as_bytes())?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

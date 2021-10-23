use anyhow::{bail, Result};
use ass_lib::ShaderIR;
use ass_types::{VertexField, VertexInput};
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
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Shader {
    pub fragment_shader: String,
    pub vertex_shader: String,
    pub texture_name: String,
    pub uniform_name: String,
    pub vertex_input: VertexInput,
}
impl Shader {
    const EXTENSION: &'static str = "ass_glsl";
    fn get_uniform_name(
        ir: &ShaderIR,
        reflection: &naga::back::glsl::ReflectionInfo,
        options: &Options,
    ) -> Result<String> {
        if options.verbose {
            println!("processing uniforms:");
            for (handle, uniform_name) in reflection.uniforms.iter() {
                println!(
                    "variable: {:#?}\nname: {}",
                    ir.module.global_variables[*handle], uniform_name
                );
            }
        }
        if reflection.uniforms.len() != 1 {
            bail!(
                "there must be 1 uniform got {} uniforms",
                reflection.uniforms.len()
            );
        }
        Ok(reflection.uniforms.iter().next().unwrap().1.to_string())
    }
    fn get_texture_name(
        reflection: &naga::back::glsl::ReflectionInfo,
        _options: &Options,
    ) -> Result<String> {
        let texture_vec = reflection
            .texture_mapping
            .iter()
            .map(|(name, _tex)| name)
            .cloned()
            .collect::<Vec<_>>();
        if texture_vec.len() == 0 {
            bail!("there are zero textures in shader there must be one texture")
        }
        if texture_vec.len() > 1 {
            bail!(
                "there are {} textures there must only be one",
                texture_vec.len()
            )
        }
        Ok(texture_vec[0].to_string())
    }
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
        let (fragment_shader, frag_info) =
            Self::write_string(&ir, &options, ShaderStage::Fragment)?;
        if options.verbose {
            println!(
                "reflection info:{{\n\ttexture_mapping: {:#?}\n\tuniforms: {:#?}\n}}",
                frag_info.texture_mapping, frag_info.uniforms
            );
        }
        let texture_name = Self::get_texture_name(&frag_info, &options)?;

        if options.verbose {
            println!("fragment shader:\n{}", fragment_shader)
        }
        let (vertex_shader, vert_info) = Self::write_string(&ir, &options, ShaderStage::Vertex)?;

        if options.verbose {
            println!("vertex shader:\n{}", vertex_shader)
        }
        let vertex_input = ir.get_vertex_input()?;
        let uniform_name = Self::get_uniform_name(&ir, &vert_info, &options)?;

        Ok(Self {
            fragment_shader,
            vertex_shader,
            vertex_input,
            texture_name,
            uniform_name,
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
    /// Reads from json str, errors if parse
    /// is unsucessfull
    pub fn from_json_str(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
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

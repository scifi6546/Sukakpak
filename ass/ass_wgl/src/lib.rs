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
    /// flattens uniform struct to make struct work better with webgl
    fn flatten_uniform_struct(mut ir: ShaderIR, options: &Options) -> Result<ShaderIR> {
        let handle_option = {
            ir.module
                .global_variables
                .iter()
                .filter(|(_handle, var)| match var.class {
                    naga::StorageClass::Uniform => true,
                    _ => false,
                })
                .next()
                .clone()
        };
        if handle_option.is_none() {
            bail!("no uniforms in shader");
        }
        let (uniform_handle, uniform) = handle_option.unwrap().clone();
        println!("uniform type: \n{:#?}", ir.module.types[uniform.ty]);
        let members = match &ir.module.types[uniform.ty].inner {
            naga::TypeInner::Struct { members, .. } => members.clone(),
            _ => bail!("invalid uniform type"),
        };
        for mem in members.iter() {
            println!("struct member:\n {:#?}", mem);
            println!("struct member type:\n {:#?}", ir.module.types[mem.ty]);
            ir.module.global_variables.append(
                naga::GlobalVariable {
                    name: Some(uniform.name.unwrap().clone() + "_" + &mem.name.unwrap()),
                    class: naga::StorageClass::Uniform,
                    binding: None,
                    ty: mem.ty,
                    init: None,
                },
                Default::default(),
            );
        }
        for function in ir
            .module
            .functions
            .iter()
            .map(|(_handle, function)| function)
            .chain(
                ir.module
                    .entry_points
                    .iter()
                    .map(|entry_point| &entry_point.function),
            )
        {
            let uniform_refs = function
                .expressions
                .iter()
                .filter_map(|(handle, exp)| match exp {
                    naga::Expression::GlobalVariable(var) => Some((handle, var)),
                    _ => None,
                })
                .filter_map(|(handle, var)| {
                    if *var == uniform_handle {
                        Some(handle)
                    } else {
                        None
                    }
                });
            for uni in uniform_refs {
                println!("{:#?}", uni);
            }
        }
        let uniform_members = ir
            .module
            .global_variables
            .iter()
            .filter(|(_handle, var)| match var.class {
                naga::StorageClass::Uniform => true,
                _ => false,
            })
            .filter(
                |(_handle, var)| match ir.module.types.get_handle(var.ty).unwrap().inner {
                    naga::TypeInner::Struct { .. } => true,
                    _ => false,
                },
            )
            .map(
                |(handle, var)| match &ir.module.types.get_handle(var.ty).unwrap().inner {
                    naga::TypeInner::Struct { members, .. } => (handle, members.clone()),
                    _ => panic!("invalid state"),
                },
            )
            .collect::<Vec<_>>();
        let num_functions = ir.module.functions.len();
        for f in ir
            .module
            .functions
            .iter()
            .map(|(_handle, function)| function)
            .chain(
                ir.module
                    .entry_points
                    .iter()
                    .map(|entry_point| &entry_point.function),
            )
        {
            println!("expressions ");
            for exp in f.expressions.iter() {
                println!("{:#?}", exp);
            }
        }
        let expressions = ir
            .module
            .functions
            .iter()
            .map(|(_handle, function)| function)
            .chain(
                ir.module
                    .entry_points
                    .iter()
                    .map(|entry_point| &entry_point.function),
            )
            .map(|function| function.expressions.iter())
            .flatten()
            .filter_map(|(_handle, expression)| match expression {
                naga::Expression::GlobalVariable(var) => Some(var),
                _ => None,
            })
            .collect::<Vec<_>>();
        if options.verbose {
            println!("number of uniforms: {}", uniform_members.len());
            for member in uniform_members.iter() {
                println!("{:#?}", member);
            }
            println!("num functions: {}", num_functions);
            println!("num entrypoints: {}", ir.module.entry_points.len());
            println!("num expressions to change: {}", expressions.len());
            for exp in expressions.iter() {
                println!(
                    "variable: {:#?}",
                    ir.module.global_variables.try_get(**exp).unwrap()
                );
            }
        }
        todo!()
    }
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
        let ir = Self::flatten_uniform_struct(ir, &options)?;
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

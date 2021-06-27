use anyhow::Result;
use serde::Deserialize;
use serde_json;
use std::{fs::File, io::Read, path::Path};
pub struct Shader {
    vertex_shader: Module,
}
use naga::{front::glsl, Module};
#[derive(Deserialize, Debug)]
pub struct ShaderConfig {
    vertex_shader: ModuleConfig,
}
#[derive(Deserialize, Debug)]
pub struct ModuleConfig {
    entry_point: EntryPoint,
}
#[derive(Deserialize, Debug)]
pub struct EntryPoint {
    name: String,
    stage: ShaderStage,
}
#[derive(Deserialize, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}
pub fn load_directory(path: &Path) -> Result<Shader> {
    let config: ShaderConfig = {
        let config_file = File::open(path.join("config.json"))?;
        serde_json::from_reader(config_file)?
    };
    let mut file = File::open(path.join("shader.vert"))?;
    let mut frag_str = String::new();
    file.read_to_string(&mut frag_str)?;
    let vertex_shader = glsl::parse_str(
        &frag_str,
        &glsl::Options {
            entry_points: naga::FastHashMap::default(),
            defines: naga::FastHashMap::default(),
        },
    )?;
    Ok(Shader { vertex_shader })
}
#[cfg(test)]
mod tests {
    const PUSH: &str = "#version 450
    layout(push_constant) uniform constants{
        mat4 proj;
    } ubo;
    layout(location=0) in vec3 pos;
    layout(location=1) in vec2 uv;
    layout(location=0) out vec2 o_uv;
    void main(){
        gl_Position = ubo.proj*vec4(pos,1.0);
        o_uv = uv;
    }
    ";
    #[test]
    fn it_works() {
        naga::front::glsl::parse_str(
            PUSH,
            &naga::front::glsl::Options {
                entry_points: naga::FastHashMap::default(),
                defines: naga::FastHashMap::default(),
            },
        )
        .expect("parsed");
        assert_eq!(2 + 2, 4);
    }
}

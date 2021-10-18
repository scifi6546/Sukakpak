use ass_lib::{
    anyhow::{Context, Result},
    ShaderIR,
};
use std::path::PathBuf;
use structopt::StructOpt;
/// output as vulkan
fn output_vulkan(ir: ShaderIR, options: CommandlineOptions) -> Result<()> {
    let vulkan = ass_vk::Shader::from_ir(
        ir,
        ass_vk::Options {
            verbose: options.verbose,
        },
    )?;
    vulkan.write_to_disk(options.out_file)?;
    if let Some(p) = options.vertex_spv {
        vulkan
            .write_vertex_to_disk(p)
            .with_context(|| "failed to write spv to disk")?;
    }
    if let Some(p) = options.fragment_spv {
        vulkan
            .write_fragment_to_disk(p)
            .with_context(|| "failed to write spv to disk")?;
    }

    Ok(())
}
fn output_wgl(ir: ShaderIR, options: CommandlineOptions) -> Result<()> {
    let glsl = ass_wgl::Shader::from_ir(
        ir,
        ass_wgl::Options {
            verbose: options.verbose,
        },
    )?;
    glsl.write_to_disk(options.out_file)
}
fn run(options: CommandlineOptions) -> Result<()> {
    let ir = ShaderIR::compile_from_disk(
        options.input_file.clone(),
        ass_lib::Options {
            verbose: options.verbose,
        },
    )?;
    match options.output_type {
        OutputType::Vk => output_vulkan(ir, options),
        OutputType::Wgl => output_wgl(ir, options),
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Vk,
    Wgl,
}
impl std::str::FromStr for OutputType {
    type Err = String;
    fn from_str(str_in: &str) -> std::result::Result<Self, String> {
        let lowercase = str_in.to_lowercase();
        if &lowercase == "vk" {
            Ok(Self::Vk)
        } else if &lowercase == "wgl" {
            Ok(Self::Wgl)
        } else {
            Err(format!(
                "invalid output type \"{}\", valid output types are \"vk\" and \"wgl\"",
                str_in
            ))
        }
    }
}
#[derive(Debug, StructOpt)]
#[structopt(name = "ass", about = "Sukakpak shader compiller")]
struct CommandlineOptions {
    /// Input Shader Directory
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,
    /// Where to write saved file
    #[structopt(parse(from_os_str))]
    out_file: PathBuf,
    /// Verbose output
    #[structopt(short = "V", long = "verbose")]
    verbose: bool,
    #[structopt(long = "vertex-spv", parse(from_os_str))]
    vertex_spv: Option<PathBuf>,
    #[structopt(long = "fragment-spv", parse(from_os_str))]
    fragment_spv: Option<PathBuf>,
    ///Format of output
    #[structopt(short = "o", long = "output-type")]
    output_type: OutputType,
}
fn main() {
    let opt = CommandlineOptions::from_args();
    run(opt).expect("failed to compile shader");
}

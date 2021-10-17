use ass_lib_v2::{
    anyhow::{Context, Result},
    ShaderIR,
};
use std::path::PathBuf;
use structopt::StructOpt;
fn run(options: Opt) -> Result<()> {
    let ir = ShaderIR::compile_from_disk(
        options.input_file,
        ass_lib_v2::Options {
            verbose: options.verbose,
        },
    )?;
    let vulkan = ass_lib_v2::vk::Shader::from_ir(ir)?;
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
#[derive(Debug, StructOpt)]
#[structopt(name = "ass", about = "Sukakpak shader compiller")]
struct Opt {
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
}
fn main() {
    let opt = Opt::from_args();
    run(opt).expect("failed to compile shader");
}

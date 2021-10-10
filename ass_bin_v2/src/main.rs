use ass_lib_v2::{anyhow::Result, ShaderIR};
use std::path::PathBuf;
use structopt::StructOpt;
fn run(input_path: PathBuf, output_path: PathBuf, verbose: bool) -> Result<()> {
    let ir = ShaderIR::compile_from_disk(input_path, ass_lib_v2::Options { verbose })?;
    let vulkan = ass_lib_v2::vk::Shader::from_ir(ir)?;
    vulkan.write_to_disk(output_path)?;

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
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}
fn main() {
    let opt = Opt::from_args();
    run(opt.input_file, opt.out_file, opt.verbose).expect("failed to compile shader");
}

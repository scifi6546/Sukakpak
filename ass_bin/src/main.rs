use ass_lib::{load_directory, AssembledSpirv};
use clap::{App, Arg};
use std::{convert::TryInto, path::Path};
fn main() {
    let matches = App::new("Sukakpak Shader Assembler")
        .version("0.1")
        .author("Skookum")
        .about("Assembles shader directory into a Skukakpak Shader")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("directory to load from")
                .required(true),
        )
        .get_matches();
    let path = matches.value_of("path").unwrap();
    let shader = load_directory(Path::new(path)).expect("failed to load directory");
    let assembled: AssembledSpirv = shader.try_into().expect("");
}

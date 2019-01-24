use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    pub src: PathBuf,
    #[structopt(parse(from_os_str))]
    pub dst: PathBuf,
}

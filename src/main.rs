mod loader;
mod utils;
mod spmc;
mod loader_async;

use std::ffi::OsStr;
use std::path::PathBuf;

use clap::Clap;

fn from_os(v: &OsStr) -> String {
    String::from(v.to_str().unwrap())
}


#[derive(Clap)]
#[clap(version = "1.0", author = "koder")]
struct Opts {
    #[clap(short="m", long, default_value="5")]
    max_level: usize,

    #[clap(parse(from_os_str=from_os))]
    initial_url: String,

    #[clap(parse(from_os_str))]
    save_dir: PathBuf,

    #[clap(short="t", long, default_value="10")]
    th_count: usize,
}

#[allow(dead_code)]
fn format_first_n(v: Vec<String>, count: usize) -> String {
    v.into_iter()
        .take(count)
        .map(|v| format!("\n    {}", v))
        .collect::<Vec<String>>()
        .concat()
}

fn main() {
    let opts: Opts = Opts::parse();
    loader::load_loop_mt(&opts.initial_url, &opts.save_dir, opts.max_level, opts.th_count);
}


use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// The file to parse as the .crates2.json if you don't want to use the default one
    #[arg(short = 'c', long = "crate-file")]
    pub crates_file: Option<PathBuf>,
    /// Change the target of all to-be-installed packges to match this device's target if it's different
    #[arg(short = 'f', long = "fix-target", default_value_t = true)]
    pub fix_target: bool,
    /// Whether to install the latest version available (true) or install the version in the
    /// lockfile (false, default)
    #[arg(short = 'i', long = "install-latest", default_value_t = false)]
    pub install_latest: bool,
    /// Whether to enable verbose output
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    pub verbose: bool,
}

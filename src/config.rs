use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
	#[command(subcommand, name = "restore")]
	pub cmd: Option<RestoreCommand>,
	#[command(flatten)]
	pub opts: SharedOptions,
}

#[derive(Subcommand, Debug)]
pub enum RestoreCommand {
	Restore(SharedOptions),
}

#[derive(Args, Debug, Clone)]
pub struct SharedOptions {
	/// The file to parse as the .crates2.json if you don't want to use the default one
	#[arg(short = 'c', long = "crate-file")]
	pub crates_file: Option<PathBuf>,
	/// Change the target of all to-be-installed packges to match this device's target if it's different
	#[arg(short = 't', long = "fix-target", default_value_t = true)]
	pub fix_target: bool,
	/// Whether to install the latest version available (true) or install the version in the
	/// lockfile (false, default)
	#[arg(short = 'i', long = "install-latest", default_value_t = false)]
	pub install_latest: bool,
	/// Whether to force (re)installation of al packages listed in .crates2.json, even if we can
	/// detect that all their binaries are already correctly installed
	#[arg(short = 'f', long = "force-all", default_value_t = false)]
	pub force_all: bool,
	/// Whether to enable verbose output
	#[arg(short = 'v', long = "verbose", default_value_t = false)]
	pub verbose: bool,
}

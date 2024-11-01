use std::{
	collections::{BTreeMap, BTreeSet},
	io::Read,
	path::PathBuf
};

use cargo::{core::package_id::PackageId, util::Filesystem, CargoResult};
use serde::Deserialize;
use yoke::{Yoke, Yokeable};

use crate::config;

// stolen from https://doc.rust-lang.org/1.69.0/nightly-rustc/cargo/ops/common_for_install_and_uninstall
#[derive(Debug)]
pub struct CrateData {
	pub root: Filesystem,
	pub listing: Yoke<CrateListingV2<'static>, Vec<u8>>
}

#[derive(Deserialize, Yokeable, Debug)]
pub struct CrateListingV2<'info> {
	#[serde(borrow)]
	pub installs: BTreeMap<PackageId, InstallInfo<'info>>
}

// This has more fields but we don't use them so we can ignore them
#[derive(Deserialize, Debug)]
pub struct InstallInfo<'info> {
	#[serde(borrow)]
	pub bins: BTreeSet<&'info str>,
	#[serde(borrow)]
	pub features: BTreeSet<&'info str>,
	pub all_features: bool,
	pub no_default_features: bool,
	#[serde(borrow)]
	pub profile: &'info str,
	#[serde(borrow)]
	pub target: Option<&'info str>
}

// Stolen from cargo::ops::common_for_install_and_uninstall
fn resolve_root(config: &cargo::GlobalContext) -> CargoResult<Filesystem> {
	let config_root = config.get_path("install.root")?;
	Ok(config
		.get_env_os("CARGO_INSTALL_ROOT")
		.map(PathBuf::from)
		.or_else(move || config_root.map(|v| v.val))
		.map(Filesystem::new)
		.unwrap_or_else(|| config.home().clone()))
}

pub fn load_info(opts: &config::SharedOptions) -> CargoResult<CrateData> {
	let mut data = vec![];
	let cargo_config = cargo::GlobalContext::default()?;
	let root = resolve_root(&cargo_config)?;

	match opts.crates_file {
		Some(ref cf) => data = std::fs::read(cf)?,
		None => {
			let lock = root.open_ro_shared(
				".crates2.json",
				&cargo_config,
				"Reading list of installed crates"
			)?;
			lock.file().read_to_end(&mut data)?;
		}
	}

	let listing = Yoke::try_attach_to_cart(data, |slice| serde_json::from_slice(slice))?;

	Ok(CrateData { root, listing })
}

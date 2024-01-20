use cargo::{
	core::{
		compiler::{CompileKind, CompileTarget},
		resolver::CliFeatures,
		FeatureValue, Verbosity,
	},
	ops::{install, CompileFilter, CompileOptions, FilterRule, LibRule, Packages},
	util::{command_prelude::CompileMode, interning::InternedString},
};
use clap::Parser;
use semver::{Comparator, VersionReq};
use std::{collections::BTreeSet, rc::Rc};

mod config;
mod crates;

fn main() {
	// maybe fix some compilation issues
	std::env::set_var("CARGO_CACHE_RUSTC_INFO", "1");

	let config = config::Config::parse();
	let opts = config
		.cmd
		.map(|cmd| match cmd {
			config::RestoreCommand::Restore(opts) => opts,
		})
		.unwrap_or(config.opts);

	let crates = crates::load_info(&opts).expect("Couldn't load list of crates");
	let listing = crates.listing.get();
	let bin_dir = crates.root.clone().join("bin");

	let install_check_conf = cargo::Config::default().expect("Couldn't create cargo Config");
	let to_install = listing
		.installs
		.iter()
		.filter(|(_, info)| {
			if opts.force_all {
				return true;
			}

			// this should be true if any of the bins don't exist so that cargo doesn't stop us
			// when it sees that the pacakge is already installed according to the lockfile
			info.bins.iter().any(|bin| {
				bin_dir
					.open_ro_shared(bin, &install_check_conf, "Checking if binary exists")
					.map_or(true, |file| !file.path().exists())
			})
		})
		.collect::<Vec<_>>();

	let mut shell = install_check_conf.shell();
	// ugh. who cares if we don't print
	if to_install.is_empty() {
		_ = shell.status(
			"All already installed!",
			"(run with -f to force re-installation of all)",
		);
		return;
	} else {
		_ = shell.status("Installing", format!("{} package(s)", to_install.len()));
	};

	let verbosity = if opts.verbose {
		for (pkg, info) in to_install.iter() {
			_ = shell.status(
				"=>",
				format!(
					"{} v{} ({}), profile: {}, features: {:?}",
					pkg.name(),
					pkg.version(),
					pkg.source_id(),
					info.profile,
					info.features
				),
			);
		}
		Verbosity::Verbose
	} else {
		Verbosity::Normal
	};

	// nice little newline
	_ = shell.print_ansi_stdout(b"\n");

	let (success, failures): (Vec<_>, Vec<_>) = to_install
		.into_iter()
		.map(|(package, info)| {
			// We want to recreate this with every package because it seems that if you use it with
			// multiple installs it can get messed up and affect later installs
			// However, this kinda irks me. The whole "it's immutable so we can assume there will be no
			// side effects" is a really nice thing about rust and it seems they're skirting it here
			// for convenience, which like goes against some core principles of rust?
			let cargo_config = cargo::Config::default().expect("Couldn't create cargo Config");
			cargo_config.shell().set_verbosity(verbosity);

			// todo)) sometimes the version can be like 0.1.0-master and the `master` is only contained
			// in the `semver::Version`, but i don't know if we can translate that over to the
			// `semver::VersionReq`. maybe it'll be fine.
			let vers = (!opts.install_latest).then(|| {
				let pkg_vers = package.version();
				VersionReq {
					comparators: vec![Comparator {
						op: semver::Op::Exact,
						major: pkg_vers.major,
						minor: Some(pkg_vers.minor),
						patch: Some(pkg_vers.patch),
						pre: pkg_vers.pre.clone(),
					}],
				}
			});

			let mut compile_opts = CompileOptions::new(&cargo_config, CompileMode::Build)
				.expect("Couldn't create compile opts");

			if opts.fix_target {
				compile_opts.build_config.requested_kinds = vec![CompileKind::Host];
			} else if let Some(target) = info.target.map(CompileTarget::new) {
				match target {
					Ok(t) => {
						compile_opts.build_config.requested_kinds = vec![CompileKind::Target(t)]
					}
					Err(e) => {
						return (
							package,
							Err(anyhow::anyhow!(
								"target specified for {} ({}) is not valid on this machine: {e}",
								package.name(),
								info.target.unwrap_or("None")
							)),
						);
					}
				}
			}

			compile_opts.build_config.force_rebuild = true;
			compile_opts.build_config.requested_profile = InternedString::new(info.profile);
			compile_opts.cli_features = CliFeatures {
				features: Rc::new(BTreeSet::from_iter(
					info.features
						.iter()
						.map(|feat| FeatureValue::Feature(InternedString::new(feat))),
				)),
				all_features: info.all_features,
				uses_default_features: !info.no_default_features,
			};

			let packages = info.bins.iter().map(|s| s.to_string()).collect::<Vec<_>>();

			compile_opts.spec = Packages::Packages(packages);
			compile_opts.filter = CompileFilter::Only {
				all_targets: false,
				lib: LibRule::Default,
				bins: FilterRule::All,
				examples: FilterRule::Just(vec![]),
				tests: FilterRule::Just(vec![]),
				benches: FilterRule::Just(vec![]),
			};

			let res = install(
				&cargo_config,
				None,
				vec![(package.name().as_str().into(), vers)],
				package.source_id(),
				false,
				&compile_opts,
				true,
				false,
			);

			if let Err(ref e) = res {
				eprintln!("Couldn't install {}: {e}", package.name());
				if opts.quick_fail {
					std::process::exit(1);
				}
			}

			(package, res)
		})
		.partition(|(_, r)| r.is_ok());

	let success_list = success
		.into_iter()
		.map(|(p, _)| p.name())
		.collect::<Vec<_>>()
		.join(", ");
	let success_str = format!("{success_list} installed successfully");

	if failures.is_empty() {
		_ = shell.status("All Succeeded!", success_str);
	} else {
		_ = shell.status("Successes", success_str);
		_ = shell.status("Failures", failures.len());
		for (package, err) in failures {
			_ = shell.status(
				"=>",
				format!(
					"{}: {}",
					package.name(),
					match err {
						Err(e) => e,
						// We should've already checked that they're all errors
						_ => unreachable!(),
					}
				),
			);
		}
	}
}

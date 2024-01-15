use cargo::{
    core::{compiler::CompileKind, resolver::CliFeatures, FeatureValue},
    ops::{install, CompileFilter, CompileOptions, FilterRule, LibRule, Packages},
    util::{command_prelude::CompileMode, interning::InternedString},
};
use clap::Parser;
use semver::{Comparator, VersionReq};
use std::{collections::BTreeSet, rc::Rc};

mod config;
mod crates;

fn main() {
    // todo)) add like tracing and stuff to make sure it looks pretty and only outputs the
    // necessary info
    let config = config::Config::parse();

    let crates = crates::load_info(&config).expect("Couldn't load list of crates");
    let listing = crates.listing.get();
    let bin_dir = crates.root.clone().join("bin");

    for (package, info) in listing.installs.iter() {
        // todo)) sometimes the version can be like 0.1.0-master and the `master` is only contained
        // in the `semver::Version`, but i don't know if we can translate that over to the
        // `semver::VersionReq`. maybe it'll be fine.
        let vers = (!config.install_latest).then(|| {
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

        let mut opts = CompileOptions::new(&crates.config, CompileMode::Build)
            .expect("Couldn't create compile opts");

        if config.fix_target {
            opts.build_config.requested_kinds = vec![CompileKind::Host];
        }

        opts.build_config.requested_profile = InternedString::new(info.profile);
        opts.cli_features = CliFeatures {
            features: Rc::new(BTreeSet::from_iter(
                info.features
                    .iter()
                    .map(|feat| FeatureValue::Feature(InternedString::new(feat))),
            )),
            all_features: info.all_features,
            uses_default_features: !info.no_default_features,
        };

        let packages = info.bins.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        opts.spec = Packages::Packages(packages);
        opts.filter = CompileFilter::Only {
            all_targets: false,
            lib: LibRule::Default,
            bins: FilterRule::All,
            examples: FilterRule::Just(vec![]),
            tests: FilterRule::Just(vec![]),
            benches: FilterRule::Just(vec![]),
        };

        // force_build should be true if any of the bins don't exist so that cargo doesn't stop us
        // when it sees that the pacakge is already installed according to the lockfile
        let force_build = info.bins.iter().any(|bin| {
            bin_dir
                .open_ro_shared(bin, &crates.config, "Checking if binary exists")
                .map_or(true, |file| !file.path().exists())
        });

        // todo)) find a way to make this quieter
        let res = install(
            &crates.config,
            None,
            vec![(package.name().as_str().into(), vers)],
            package.source_id(),
            false,
            &opts,
            force_build,
            false,
        );

        if let Err(e) = res {
            eprintln!("Couldn't install {}: {e}", package.name());
        }
    }
}

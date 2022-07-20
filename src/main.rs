use std::env;
use std::path::Path;
use std::process::Command;

use crate::args::{Args, Operation};
use crate::internal::AppExitCode;
use crate::repository::create_config;
use clap::Parser;

use crate::workspace::read_cfg;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod args;
mod internal;
mod operations;
mod repository;
mod workspace;

fn main() {
    if unsafe { libc::geteuid() } == 0 {
        crash!(AppExitCode::RunAsRoot, "Running malachite as root is disallowed as it can lead to system breakage. Instead, malachite will prompt you when it needs superuser permissions");
    }

    let args: Args = Args::parse();

    if Path::exists("mlc.toml".as_ref()) && Path::exists(".git".as_ref()) {
        info!(
            "In a git repository, pulling latest mlc.toml. It is advised you run mlc pull/update"
        );
        Command::new("git")
            .arg("pull")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    if Path::exists("../.git".as_ref()) {
        info!("Parent directory is a git directory, pulling latest mlc.toml. It is advised you run mlc pull/update in all malachite directories");
        let dir = env::current_dir().unwrap();
        env::set_current_dir("../").unwrap();
        Command::new("git")
            .arg("pull")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        env::set_current_dir(dir).unwrap();
    }

    match args.subcommand.unwrap_or(Operation::Init) {
        Operation::Init => operations::init(),
        Operation::Build {
            packages,
            exclude,
            no_regen,
            ..
        } => operations::build(packages, exclude, no_regen),
        Operation::Pull {
            packages, exclude, ..
        } => operations::pull(packages, exclude),
        Operation::RepoGen => {
            let config = read_cfg();
            if config.mode != "repository" {
                crash!(
                    AppExitCode::BuildInWorkspace,
                    "Cannot build packages in workspace mode"
                )
            }
            info!("Generating repository: {}", config.name.unwrap());
            repository::generate();
        }
        Operation::Prune => operations::prune(),
        Operation::Config => operations::config(),
    }
}

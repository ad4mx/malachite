use std::env;
use std::path::Path;
use std::process::Command;

use crate::internal::{crash, info};
use clap::{App, AppSettings, Arg, ArgSettings, SubCommand};
use crate::repository::create_config;

use crate::workspace::read_cfg;

mod internal;
mod repository;
mod workspace;

fn main() {
    fn build_app() -> App<'static, 'static> {
        let app = App::new("Malachite")
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .multiple(true)
                    .set(ArgSettings::Global)
                    .help("Sets the level of verbosity"),
            )
            .arg(
                // TODO implement --exclude
                Arg::with_name("exclude")
                    .short("e")
                    .long("exclude")
                    .multiple(true)
                    .set(ArgSettings::Global)
                    .help("Excludes packages from given operation"),
            )
            .arg(
                // TODO implement --all
                Arg::with_name("all")
                    .long("all")
                    .set(ArgSettings::Global)
                    .help("Operates on every possible package"),
            )
            .subcommand(
                SubCommand::with_name("build")
                    .about("Builds the given packages")
                    .arg(
                        Arg::with_name("package(s)")
                            .help("The packages to operate on")
                            .required(true)
                            .multiple(true)
                            .index(1),
                    ),
            )
            .subcommand(
                SubCommand::with_name("repo-gen").about("Generates repository from built packages"),
            )
            .subcommand(
                SubCommand::with_name("prune")
                    .about("Prunes duplicate packages older than X days from the repository")
                    .arg(
                        Arg::with_name("days")
                            .help("How old a duplicate package needs to be (in days) to be pruned")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(SubCommand::with_name("init").about(
                "Clones all git repositories from mlc.toml branching from current directory",
            ))
            .subcommand(
                SubCommand::with_name("pull").alias("update").about(
                    "Pulls all git repositories from mlc.toml branching from current directory",
                ),
            )
            .subcommand(
                SubCommand::with_name("config").about("Create and/or open local config file"),
            )
            .settings(&[
                AppSettings::GlobalVersion,
                AppSettings::VersionlessSubcommands,
                AppSettings::ArgRequiredElseHelp,
                AppSettings::InferSubcommands,
            ]);
        app
    }

    let matches = build_app().get_matches();

    if let true = matches.is_present("init") {
        let config = workspace::read_cfg();
        if config.mode == "workspace" {
            for r in config.repo {
                info(format!("Cloning (workspace mode): {}", r));
                Command::new("git")
                    .args(&["clone", &r])
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        } else if config.mode == "repository" {
            for r in config.repo {
                info(format!("Cloning (repository mode): {}", r));
                Command::new("git")
                    .args(&["clone", "--no-checkout", &r])
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();

                info(format!("Entering working directory: {}", r));
                let dir = format!(
                    "{}/{}",
                    env::current_dir().unwrap().display(),
                    r.split('/').collect::<Vec<&str>>().last().unwrap()
                );
                env::set_current_dir(dir).unwrap();

                info(format!("Resetting unstaged files: {}", r));
                Command::new("git")
                    .arg("reset")
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();

                info(format!("Checking out PKGBUILD: {}", r));
                Command::new("git")
                    .args(&["checkout", "HEAD", "PKGBUILD"])
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        } else {
            crash("Invalid mode in mlc.toml".to_string(), 1);
        }
    }

    if let true = matches.is_present("build") {
        let config = workspace::read_cfg();
        if config.mode != "repository" {
            crash("Cannot build packages in workspace mode".to_string(), 2);
        }
        let packages: Vec<String> = matches
            .subcommand()
            .1
            .unwrap()
            .values_of("package(s)")
            .unwrap()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let mut repos: Vec<String> = vec![];
        for r in config.repo {
            let split = r.split('/').collect::<Vec<&str>>();
            let a = split.last().unwrap();
            repos.push(a.parse().unwrap());
        }

        for pkg in packages {
            if !repos.contains(&pkg) {
                crash(format!("Package {} not found in repos in mlc.toml", pkg), 3);
            } else {
                repository::build(pkg);
            }
        }
    }

    if let true = matches.is_present("pull") {
        let config = workspace::read_cfg();
        let cdir = env::current_dir().unwrap();
        for r in config.repo {
            info(format!("Entering working directory: {}", r));
            let dir = format!(
                "{}/{}",
                env::current_dir().unwrap().display(),
                r.split('/').collect::<Vec<&str>>().last().unwrap()
            );
            env::set_current_dir(dir).unwrap();
            Command::new("git")
                .args(&["pull", &r])
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
            env::set_current_dir(&cdir).unwrap();
        }
    }

    if let true = matches.is_present("repo-gen") {
        let config = read_cfg();
        if config.mode != "repository" {
            panic!("Cannot build packages in workspace mode")
        }
        repository::generate();
    }

    if let true = matches.is_present("config") {
        if !Path::exists("mlc.toml".as_ref()) {
            create_config();
        }
        let editor = env::var("EDITOR").unwrap_or("nano".to_string());
        Command::new(editor)
            .arg("mlc.toml")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
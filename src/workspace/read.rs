use std::fs;
use std::path::Path;

use crate::internal::structs::{Config, Repo, SplitRepo, UnexpandedConfig};
use crate::internal::AppExitCode;
use crate::{crash, log};

pub fn read_cfg(verbose: bool) -> Config {
    // Crash if mlc.toml doesn't exist
    if !Path::exists("mlc.toml".as_ref()) {
        crash!(
            AppExitCode::ConfigNotFound,
            "Config file not found (mlc.toml)"
        )
    }

    // Reading the config file to an UnexpandedConfig struct
    let file = fs::read_to_string("mlc.toml").unwrap();
    let config: UnexpandedConfig = toml::from_str(&file).unwrap();
    log!(verbose, "Config file read: {:?}", config);

    // Crash if incorrect mode is set
    if config.mode != "workspace" && config.mode != "repository" {
        crash!(
            AppExitCode::InvalidMode,
            "Invalid mode in mlc.toml, must be either \"repository\" or \"workspace\""
        );
    }

    let mut expanded_repos: Vec<Repo> = vec![];

    // Parsing repos from the config file
    for x in config.repo {
        log!(verbose, "Parsing repo: {:?}", x);
        // Splits the repo name and index inta a SplitRepo struct
        let split: Vec<&str> = x.split("::").collect();
        let split_struct = SplitRepo {
            indx: split[0].parse().unwrap(),
            name: split[1].parse().unwrap(),
        };
        log!(verbose, "Split repo: {:?}", split_struct);

        // Parses all necessary values for expanding the repo to a Repo struct
        let index = split_struct.indx;
        let name = split_struct.name.replace('!', "");
        let url = config.urls[index - 1].replace("%repo%", &name);
        let priority = &split_struct.name.matches('!').count();

        // Creates and pushes Repo struct to expanded_repos
        let repo = Repo {
            name,
            url,
            priority: *priority,
        };
        log!(verbose, "Expanded repo: {:?}", repo);
        expanded_repos.push(repo);
    }

    // Returns parsed config file
    let conf = Config {
        mode: config.mode,
        sign: config.sign,
        name: config.name,
        smart_pull: config.smart_pull,
        repo: expanded_repos,
    };
    log!(verbose, "Config: {:?}", conf);
    conf
}

use crate::{info, workspace};
use std::process::Command;

pub fn clone() {
    // Read config struct from mlc.toml
    let config = workspace::read_cfg();
    let repos = &config.repo;

    // Get a vector of all files/dirs in the current directory, excluding config file
    let dir_paths = std::fs::read_dir("./").unwrap();
    let mut dirs = dir_paths
        .map(|x| x.unwrap().path().display().to_string())
        .collect::<Vec<String>>();
    dirs.retain(|x| *x != "./mlc.toml");

    // Creates a vector of the difference between cloned repos and repos defined in config
    let mut repo_diff = vec![];
    for repo in repos {
        let name = &repo.name;
        if !dirs.contains(name) {
            repo_diff.push(repo);
        }
    }

    // Diff logic
    if repo_diff.is_empty() {
        // No diff, do nothing
        info!("All repos are already cloned");
    } else {
        // This is just for pretty display purposes
        let display = repo_diff
            .iter()
            .map(|x| x.name.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        info!("New/missing repos to clone: {}", display);

        // Clone all diff repos
        for r in repo_diff {
            info!("Cloning ({} mode): {}", config.mode, r.name);
            Command::new("git")
                .args(&["clone", &r.url, &r.name])
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
    }
}

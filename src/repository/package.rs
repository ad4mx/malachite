use std::{env, fs};
use std::path::Path;
use std::process::Command;

pub fn build(pkg: String) {
    let dir = env::current_dir().unwrap();
    if !Path::exists("out".as_ref()) {
        fs::create_dir_all("out").unwrap();
    }
    if !Path::exists(pkg.as_ref()) {
        panic!("Git directory for {} not found, aborting", pkg);
    }

    env::set_current_dir(pkg).unwrap();
    Command::new("updpkgsums").spawn().unwrap().wait().unwrap();

    Command::new("makepkg")
        .args(&["-sf", "--skippgpcheck", "--sign", "--noconfirm"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Command::new("bash")
        .args(&["-c", "cp *.pkg.tar.zst* ../out/"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    env::set_current_dir(dir).unwrap();
}
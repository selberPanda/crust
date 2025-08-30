use std::process::{Command, Stdio};

use crate::utils::repo_dir;

pub fn check_installed(pkgname: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", pkgname])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn clone_pkg(pkgname: &str) -> bool {
    let pkg_url = format!("https://aur.archlinux.org/{}.git", pkgname);
    let target_dir = repo_dir(pkgname);
    Command::new("git")
        .args(["clone", &pkg_url, &target_dir])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn pull_pkg() -> bool {
    let old_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse fehlgeschlagen!");
    let old_hash = String::from_utf8(old_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    Command::new("git")
        .arg("pull")
        .status()
        .expect("git pull fehlgeschlagen");

    let new_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse fehlgeschlagen!");
    let new_hash = String::from_utf8(new_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    if old_hash == new_hash {
        return false;
    } else {
        return true;
    }
}

pub fn build_pkg() -> bool {
    Command::new("makepkg")
        .arg("-si")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn remove_build_dir(pkgname: &str) -> bool {
    let target_dir = repo_dir(pkgname);
    Command::new("rm")
        .args(["-rf", &target_dir])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

mod aur;
mod cmd;
mod pkg;
mod utils;

use crate::cmd::{App, SubCommand};
use crate::pkg::{build_pkg, check_installed, clone_pkg, pull_pkg, remove_build_dir};
use crate::{aur::get_pkglist, utils::change_dir, utils::repo_dir};
use clap::Parser;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[tokio::main]
async fn main() {
    let app = App::parse();

    match app.subcommand {
        SubCommand::Install { pkgname } => install_pkg(pkgname).await,
        SubCommand::Remove { pkgname } => remove_pkg(pkgname),
        SubCommand::Update {} => update_pkg(),
    }
}

async fn install_pkg(pkgname: Vec<String>) {
    for (_i, pkgname) in pkgname.iter().enumerate() {
        let url = format!("https://aur.archlinux.org/rpc/v5/search/{}", pkgname);
        let pkglist = get_pkglist(&url).await.unwrap();
        let pkglist = pkglist.results;

        for (i, pkg) in pkglist.iter().rev().enumerate() {
            println!(
                "{}  {} {}\n{}\n",
                pkglist.len() - i,
                pkg.name.as_deref().unwrap_or("N/A"),
                pkg.version.as_deref().unwrap_or("N/A"),
                pkg.description.as_deref().unwrap_or("Keine Beschreibung")
            );
        }

        let mut pkg_input = String::new();
        println!("Wähle zu installierendes Paket aus [1],[2],... : ");
        io::stdin().read_line(&mut pkg_input).unwrap();

        let mut pkgname = String::new();

        if let Ok(num) = pkg_input.trim().parse::<usize>() {
            if num > 0 && num <= pkglist.len() {
                let pkg = &pkglist[num - 1];
                pkgname = pkg.name.clone().unwrap_or("N/A".to_string());

                println!("Du hast gewählt: {}", pkgname)
            } else {
                eprintln!("Ungültige Nummer!");
            }
        } else {
            eprintln!("Bitte eine Zahl eingeben!")
        }

        if check_installed(&pkgname) {
            print!("{} is schon installiert. Fortfahren? [j/N] ", pkgname);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.eq_ignore_ascii_case("j") {
                println!("Fortfahren...");
                remove_build_dir(&pkgname);
            } else {
                println!("Wird abgebrochen!");
                return;
            }
        }

        clone_pkg(&pkgname);
        change_dir(&repo_dir(&pkgname));
        build_pkg();
    }
}

fn remove_pkg(pkgname: Vec<String>) {
    let mut list = vec!["pacman", "-R"];
    list.extend(pkgname.iter().map(|s| s.as_str()));

    let pacman = Command::new("sudo")
        .args(list)
        .status()
        .expect("Fehler beim deinstallieren!");

    if pacman.success() {
        for pkgname in pkgname.iter() {
            if remove_build_dir(pkgname) {
                println!("{} erfolgreich entfernt!", repo_dir(pkgname))
            }
        }
    }
}

fn update_pkg() {
    let _pacman = Command::new("sudo")
        .args(["pacman", "-Sy"])
        .status()
        .expect("Pacman -Sy fehlgeschlagen");

    let output = Command::new("pacman")
        .arg("-Qm")
        .output()
        .expect("Fehler beim Auführen von pacman");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let packages: Vec<String> = stdout
        .lines()
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect();

    for pkg in &packages {
        if !Path::new(&repo_dir(pkg)).exists() {
            clone_pkg(pkg);
            println!("{} wird nach {} geklont!", pkg, repo_dir(pkg))
        }

        change_dir(&repo_dir(&pkg));
        if pull_pkg() {
            if build_pkg() {
                println!("{} erfolgreich gebaut!", pkg)
            } else {
                eprintln!("makepkg für {} fehlgeschlagen", pkg)
            }
        }
    }

    let _pacman = Command::new("sudo")
        .args(["pacman", "-Su"])
        .status()
        .expect("Pacman -Su fehlgeschlagen");
}

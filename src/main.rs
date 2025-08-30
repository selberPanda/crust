mod aur;
mod pkg;
mod utils;

use crate::pkg::{build_pkg, check_installed, clone_pkg, pull_pkg, remove_build_dir};
use crate::{aur::get_pkglist, utils::change_dir, utils::repo_dir};
use std::io::{self, Write};
use std::path::Path;
use std::{env, process::Command};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        eprintln!("Befehl zu ungenau!");
        return;
    }

    match args[1].as_str() {
        "install" => install_pkg(args.get(2)).await,
        "remove" => remove_pkg(args.get(2)),
        "update" => update_pkg(),
        _ => eprintln!("Unbekannter Befehl. Benutze install, remove oder update"),
    }
}

async fn install_pkg(pkgname: Option<&String>) {
    let pkgname = match pkgname {
        Some(pkgname) => pkgname,
        None => {
            eprintln!("Bitte gib einen Paketnamen an!");
            return;
        }
    };

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

fn remove_pkg(pkgname: Option<&String>) {
    let pkgname = match pkgname {
        Some(pkgname) => pkgname,
        None => {
            eprintln!("Bitte gib einen Paketnamen an!");
            return;
        }
    };

    let pacman = Command::new("sudo")
        .args(["pacman", "-R", pkgname])
        .status()
        .expect("Fehler beim deinstallieren!");

    if pacman.success() {
        if remove_build_dir(pkgname) {
            println!("{} erfolgreich entfernt!", repo_dir(pkgname))
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

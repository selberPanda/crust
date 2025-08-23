use std::{env, path::Path, process::Command};

use regex::Regex;

fn change_dir(target: &str) {
    let new_dir = Path::new(&target);

    if let Err(e) = env::set_current_dir(&new_dir) {
        eprintln!("Fehler beim Wechseln des Verzeichnisses: {}", e);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Alle Argumente: {:?}", args);

    if args.len() > 2 {
        let command = &args[1];
        let pkg = &args[2];

        match command.as_str() {
            "install" => {
                let re =
                    Regex::new(r"^https://aur\.archlinux\.org/([a-zA-Z0-9._+-]+)\.git$").unwrap();
                if let Some(caps) = re.captures(pkg) {
                    let pkgname = &caps[1];
                    let target_dir = format!("repos/{}", pkgname);

                    if Path::new(&target_dir).exists() {
                        eprintln!("Paket {} ist schon geklont!", pkgname);
                        return;
                    }

                    println!("Klonen von {} nach {}", pkg, target_dir);

                    let status = Command::new("git")
                        .args(["clone", pkg, &target_dir])
                        .status()
                        .expect("Fehler beim Ausführen von git");

                    if status.success() {
                        println!("{} erfolgreich geklont!", pkgname);

                        change_dir(&target_dir);

                        let status = Command::new("makepkg")
                            .arg("-si")
                            .status()
                            .expect("Fehler beim ausführen von makepkg");

                        if status.success() {
                            println!("{} erfolgreich gebaut!", pkgname)
                        } else {
                            eprintln!("makepkg für {} fehlgeschlagen", pkgname)
                        }
                    } else {
                        eprintln!("git clone für {} fehlgeschlagen!", pkgname);
                    }
                } else {
                    println!(
                        "Bitte gib einen gültigen AUR-Link an (z. B. https://aur.archlinux.org/pkgname.git)!"
                    );
                }
            }
            "remove" => {
                return;
            }
            _ => {
                eprintln!(
                    "Unbekannter Befehl: {}. Benutze install oder remove",
                    command
                )
            }
        }
    } else {
        println!("Bitte gib ein AUR-Paket als Argument an!");
    }
}

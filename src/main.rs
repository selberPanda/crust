use serde::Deserialize;
use std::io::{self, Write};
use std::{env, path::Path, process::Command};

fn change_dir(target: &str) {
    let new_dir = Path::new(&target);

    if let Err(e) = env::set_current_dir(&new_dir) {
        eprintln!("Fehler beim Wechseln des Verzeichnisses: {}", e);
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Package {
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    results: Vec<Package>,
}

async fn get_pkglist(url: &str) -> Result<ApiResponse, reqwest::Error> {
    let json = reqwest::get(url).await?.json::<ApiResponse>().await?;
    Ok(json)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Alle Argumente: {:?}", args);

    if args.len() > 2 {
        let command = &args[1];
        let paket = &args[2];

        match command.as_str() {
            "install" => {
                let url = format!("https://aur.archlinux.org/rpc/v5/search/{}", paket);

                let pkglist = get_pkglist(&url).await.unwrap();
                let pkgs = pkglist.results;

                for (i, pkg) in pkgs.iter().rev().enumerate() {
                    println!(
                        "{}  {} {}\n{}\n",
                        pkgs.len() - i,
                        pkg.name.as_deref().unwrap_or("N/A"),
                        pkg.version.as_deref().unwrap_or("Keine Version gegeben"),
                        pkg.description.as_deref().unwrap_or("Keine Beschreibung")
                    );
                }

                let mut pkg_input = String::new();
                println!("Wähle zu installierendes Paket aus [1],[2],... : ");
                io::stdin().read_line(&mut pkg_input).unwrap();

                let mut pkgname = String::new();

                if let Ok(num) = pkg_input.trim().parse::<usize>() {
                    if num > 0 && num <= pkgs.len() {
                        let pkg = &pkgs[num - 1];
                        pkgname = pkg.name.clone().unwrap_or("N/A".to_string());

                        println!("Du hast gewählt: {}", pkgname);
                    } else {
                        println!("Ungültige Nummer");
                    }
                } else {
                    println!("Bitte eine Zahl eingeben!");
                }

                let home = env::var("HOME").expect("HOME nicht gesetzt");
                let target_dir = format!("{}/.crust/repos/{}", home, pkgname);

                let installed = Command::new("pacman").args(["-Q", &pkgname]).status();

                match installed {
                    Ok(status) => {
                        if status.success() {
                            print!("{} ist schon Installiert. Fortfahren? [j/N] ", pkgname);
                            io::stdout().flush().unwrap();

                            let mut input = String::new();
                            io::stdin().read_line(&mut input).unwrap();
                            let input = input.trim();

                            if input.eq_ignore_ascii_case("j") {
                                println!("Fortfahren...");
                                let _ = Command::new("rm").args(["-rf", &target_dir]).status();
                            } else {
                                println!("Wird abgebrochen");
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Fehler beim prüfen des Pakets: {}", e)
                    }
                }

                println!("Klonen von {} nach {}", pkgname, target_dir);

                let pkg_url = format!("https://aur.archlinux.org/{}.git", pkgname);

                let status = Command::new("git")
                    .args(["clone", &pkg_url, &target_dir])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pkg_get() {
        let url = "https://aur.archlinux.org/rpc/v5/search/hello";

        let pkgs = get_pkglist(&url).await.unwrap();

        let packages = pkgs.results;

        for (i, pkg) in packages.iter().rev().enumerate() {
            println!(
                "{}  {} {}\n{}\n",
                packages.len() - i,
                pkg.name.as_deref().unwrap_or("N/A"),
                pkg.version.as_deref().unwrap_or("Keine Version gegeben"),
                pkg.description.as_deref().unwrap_or("Keine Beschreibung")
            );
        }

        let mut pkg_input = String::new();
        println!("Wähle zu installierendes Paket aus [1],[2],... : ");
        io::stdin().read_line(&mut pkg_input).unwrap();

        if let Ok(num) = pkg_input.trim().parse::<usize>() {
            if num > 0 && num <= packages.len() {
                let pkgname = &packages[num - 1];
                println!(
                    "Du hast gewählt: {:#?}",
                    pkgname.name.as_deref().unwrap_or("N/A")
                );
            } else {
                println!("Ungültige Nummer");
            }
        } else {
            println!("Bitte eine Zahl eingeben!");
        }
    }
}

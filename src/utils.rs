use once_cell::sync::Lazy;
use std::{env, path::Path};

pub static HOME: Lazy<String> = Lazy::new(|| env::var("HOME").unwrap_or_default());

pub fn change_dir(target_dir: &str) {
    let new_dir = Path::new(&target_dir);

    if let Err(e) = env::set_current_dir(&new_dir) {
        eprintln!("Fehler beim Wechseln des Verzeichnisses: {}", e);
    }
}

pub fn repo_dir(pkgname: &str) -> String {
    format!("{}/.crust/repos/{}", *HOME, pkgname)
}

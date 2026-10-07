use crate::settings::Settings;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub path: String,
    pub is_dir: bool,
}

/// İç içe dizinleri eler: "/home" varsa "/home/x" ayrıca taranmaz.
fn top_level_dirs(dirs: &[String]) -> Vec<String> {
    let mut sorted = dirs.to_vec();
    sorted.sort_by_key(|d| d.len()); // kısa olan (üst dizin) önce gelsin

    let mut final_dirs: Vec<String> = Vec::new();
    for dir in sorted {
        // Path::starts_with bileşen bazlı: "/home/ab", "/home/a"nın altında sayılmaz
        if !final_dirs
            .iter()
            .any(|parent| Path::new(&dir).starts_with(parent))
        {
            final_dirs.push(dir);
        }
    }
    final_dirs
}

pub fn scan_files(app_settings: &Settings) -> Vec<Entry> {
    let final_dirs = top_level_dirs(&app_settings.search_dirs);

    final_dirs
        .par_iter()
        .flat_map(|dir| {
            WalkDir::new(dir)
                .into_iter()
                .filter_entry(|e| {
                    // Kök dizinin kendisi her zaman taranır (örn. "." veya gizli bir dizin verilmişse)
                    if e.depth() == 0 {
                        return true;
                    }
                    let file_name = e.file_name().to_string_lossy();
                    // Gizli dosyaları/klasörleri (.git, .ssh vb.) atla
                    if file_name.starts_with('.') {
                        return false;
                    }
                    !app_settings.exclude_dirs.iter().any(|ex| ex == &file_name)
                })
                .filter_map(|e| e.ok())
                .map(|e| Entry {
                    path: e.path().to_string_lossy().into_owned(),
                    // WalkDir'in tuttuğu tip bilgisi, ekstra syscall yok
                    is_dir: e.file_type().is_dir(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::top_level_dirs;

    #[test]
    fn nested_dirs_are_removed() {
        let dirs = ["/home/a/b", "/home/ab", "/home/a"].map(String::from);
        assert_eq!(top_level_dirs(&dirs), ["/home/a", "/home/ab"]);
    }
}

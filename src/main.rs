use nav::{help, scanner, scanner::Entry, settings::Settings, ui};
use std::{env, fs, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        help::print_help();
        return;
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        eprintln!("nav {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let app_settings = Settings::new().expect("Ayarlar yüklenemedi!");
    let force_scan = args.iter().any(|arg| arg == "--scan");

    let index_exists = Path::new(&app_settings.index_file).exists();
    let all_files: Vec<Entry> = if force_scan || app_settings.auto_scan || !index_exists {
        let reason = if force_scan {
            "--scan komutu"
        } else if app_settings.auto_scan {
            "auto_scan ayarı"
        } else {
            "indeks bulunamadı"
        };
        eprintln!("Tarama başlatılıyor (Sebep: {reason})...");
        perform_scan_and_save(&app_settings)
    } else {
        let data = fs::read_to_string(&app_settings.index_file).unwrap_or_default();
        match serde_json::from_str::<Vec<Entry>>(&data) {
            Ok(files) => {
                eprintln!("İndeks yükleniyor ({} girdi)...", files.len());
                files
            }
            // Eski formattaki indeks de buraya düşer ve otomatik yenilenir
            Err(_) => {
                eprintln!("İndeks dosyası bozuk veya eski formatta! Yeniden taranıyor...");
                perform_scan_and_save(&app_settings)
            }
        }
    };

    match ui::run_tui(all_files, &app_settings) {
        Ok(Some(path)) => {
            let p = Path::new(&path);
            let target = if p.is_dir() {
                p
            } else {
                p.parent().unwrap_or(p)
            };
            // stdout'a sadece yol yazılır, shell fonksiyonu bunu `cd` için kullanır
            print!("{}", target.display());
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("Hata: {e}");
            std::process::exit(1);
        }
    }
}

fn perform_scan_and_save(settings: &Settings) -> Vec<Entry> {
    let files = scanner::scan_files(settings);
    // Pretty değil: milyonlarca girdide dosya boyutu ve yükleme süresi ciddi fark eder
    if let Ok(json) = serde_json::to_string(&files)
        && let Err(e) = fs::write(&settings.index_file, json)
    {
        eprintln!("Hata: İndeks dosyası yazılamadı: {e}");
    }
    files
}

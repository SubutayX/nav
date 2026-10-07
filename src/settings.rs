use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub search_dirs_linux: Vec<String>,
    pub search_dirs_windows: Vec<String>,
    #[serde(skip_deserializing)]
    pub search_dirs: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub index_file: String,
    pub fuzzy_threshold: i64,
    pub limit: usize,
    pub auto_scan: bool,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let home_dir = dirs::home_dir().expect("Ana dizin bulunamadı!");
        let nav_dir = home_dir.join(".nav");
        let config_path = nav_dir.join("Settings.toml");
        let index_path = nav_dir.join(".nav_index.json");

        // 1. .nav klasörü yoksa oluştur
        if !nav_dir.exists() {
            let _ = std::fs::create_dir_all(&nav_dir);
        }

        // 2. Dosya yoksa istediğin içeriği oluştur
        if !config_path.exists() {
            let default_toml = format!(
                r#"# Linux/macOS için tarama dizinleri
search_dirs_linux = ["{home}"]

# Windows için tarama dizinleri
search_dirs_windows = ["{home}"]

# Hariç tutulacak klasörler
exclude_dirs = [
    "target",        
    "node_modules",  
    ".git",          
    ".idea",         
    "build",         
    "venv",          
    "cache"          
]

# İndeks dosyasının konumu
index_file = "{index}"

# Minimum eşleşme skoru (0 = kapalı). Skor kabaca eşleşen harf başına ~20 artar,
# yani 75 gibi bir değer 3 harften kısa sorguları tamamen gizler.
fuzzy_threshold = 0
# Gösterilecek sonuç sayısı (3-100)
limit = 10
# Her açılışta otomatik tarama
auto_scan = false
"#,
                home = toml_escape(&home_dir),
                index = toml_escape(&index_path),
            );

            let _ = std::fs::write(&config_path, default_toml);
            eprintln!(
                "Varsayılan ayarlar {} konumuna oluşturuldu.",
                config_path.display()
            );
        }

        // 3. Config nesnesini oluştur ve oku
        let s = Config::builder()
            .set_default(
                "search_dirs_linux",
                vec![home_dir.to_string_lossy().into_owned()],
            )?
            .set_default(
                "search_dirs_windows",
                vec![home_dir.to_string_lossy().into_owned()],
            )?
            .set_default(
                "exclude_dirs",
                vec!["target".to_string(), ".git".to_string()],
            )?
            .set_default("index_file", index_path.to_string_lossy().into_owned())?
            .set_default("fuzzy_threshold", 0)?
            .set_default("limit", 10)?
            .set_default("auto_scan", false)?
            .add_source(config::File::from(config_path).required(false))
            .build()?;

        let mut settings: Self = s.try_deserialize()?;

        // İşletim sistemi kontrolü
        if cfg!(target_os = "windows") {
            settings.search_dirs = settings.search_dirs_windows.clone();
        } else {
            settings.search_dirs = settings.search_dirs_linux.clone();
        }

        settings.limit = settings.limit.clamp(3, 100);

        Ok(settings)
    }
}

/// TOML string içinde Windows yollarındaki `\` kaçışlanmalı.
fn toml_escape(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

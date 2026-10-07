# nav 🧭

[![CI](https://github.com/SubutayX/nav/actions/workflows/ci.yml/badge.svg)](https://github.com/SubutayX/nav/actions/workflows/ci.yml)

Terminalde dosya ve klasörlere **fuzzy arama** ile ışık hızında atlamanızı sağlayan, Rust ile yazılmış küçük bir TUI aracı.

Dizinlerinizi bir kez tarar, indeksi diske kaydeder. Sonrasında birkaç harf yazıp `Enter` ile istediğiniz klasöre `cd` yaparsınız.

```
╭ nav ─────────────────────────────────────────────── 📁 Klasörler ╮
│❯ help▏                                                            │
╰───────────────────────────────────────────────────────────────────╯
╭ 4 sonuç ──────────────────────────────────────────────────────────╮
│▌ 📁 help   ~/Downloads/opt/OpenVSP                                 │
│  📁 help_vsp   ~/Downloads/opt/OpenVSP/python/openvsp/openvsp      │
│  📁 helper_scripts   ~/Belgeler/Çalışma                            │
│  📁 images   ~/Downloads/opt/OpenVSP/help                          │
╰───────────────────────────────────────────────────────────────────╯
 ↑↓ gez   ⏎ git   TAB klasör/hepsi   ESC çık
```

## Özellikler

- ⚡ **Hızlı:** Tarama ve arama [rayon](https://github.com/rayon-rs/rayon) ile paralel çalışır. Arama metni önceden hazırlandığı için tuş başına disk erişimi yapılmaz.
- 🔍 **Fuzzy arama:** `rpcli` yazmanız `Rust_Proje/cli_v1` için yeterli.
- 🎨 **Eşleşme vurgusu:** Sorgunuzla eşleşen harfler sonuç listesinde renkli ve altı çizili gösterilir.
- 🇹🇷 **Türkçe karakter desteği:** `calisma` yazınca `Çalışma` klasörü bulunur (`ğ ü ş ı ö ç` normalize edilir).
- 📁 **İki mod:** Sadece klasörler veya tüm dosyalar (`TAB` ile geçiş). Dosya seçilirse bulunduğu klasöre gidilir.
- 🙈 **Akıllı filtreleme:** Gizli dosyalar (`.git`, `.cache` …) ve `target`, `node_modules` gibi klasörler atlanır.
- 🐧🍎🪟 **Linux, macOS (Intel ve Apple Silicon) ve Windows** desteği. macOS'ta `~/Library` varsayılan olarak taranmaz.

## Kurulum

### Tek komutla (önerilen)

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/SubutayX/nav/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/SubutayX/nav/main/install.ps1 | iex
```

Betik şunları yapar:
1. İşletim sisteminize ve işlemcinize uygun hazır binary'yi [Releases](https://github.com/SubutayX/nav/releases) sayfasından indirir. Linux/macOS'ta `~/.local/bin`, Windows'ta `%LOCALAPPDATA%\nav\bin` dizinine kurar.
2. Shell'inize (fish, bash, zsh veya PowerShell) `n` fonksiyonunu ekler.

Ardından yeni bir terminal açıp `n` yazmanız yeterli.

### Cargo ile

```bash
cargo install --git https://github.com/SubutayX/nav --locked
```

Bu yöntemde shell fonksiyonunu aşağıdaki gibi elle eklemeniz gerekir.

### Shell entegrasyonu (elle kurulum)

Bir program kendi başına, çağrıldığı shell'in dizinini değiştiremez. Bu yüzden `nav` seçilen yolu stdout'a yazar ve küçük bir shell fonksiyonu bu yola `cd` yapar. Kurulum betiği bunu otomatik ekler. Elle kurduysanız shell'inize göre aşağıdakilerden birini ekleyin:

**Fish** (`~/.config/fish/functions/n.fish`):

```fish
function n
    set -l dir (nav $argv)
    and test -n "$dir"
    and cd "$dir"
end
```

**Bash / Zsh** (`~/.bashrc` veya `~/.zshrc`):

```bash
n() { local dir; dir="$(nav "$@")" && [ -n "$dir" ] && cd "$dir"; }
```

**PowerShell** (`$PROFILE`):

```powershell
function n { $dir = nav @args; if ($dir) { Set-Location $dir } }
```

## Kullanım

```bash
n            # indeksi yükle ve aramayı aç
n --scan     # dizinleri yeniden tara, sonra aramayı aç
nav --help   # yardım
```

| Tuş | İşlev |
|---|---|
| Yazmak | Fuzzy arama |
| `↑` / `↓` | Sonuçlar arasında gezinme |
| `TAB` | Klasör modu ile tüm dosyalar modu arasında geçiş |
| `Enter` | Seçilen konuma git |
| `ESC` / `Ctrl+C` | Çıkış |

## Yapılandırma

İlk çalıştırmada `~/.nav/Settings.toml` otomatik oluşturulur:

```toml
# Linux/macOS için tarama dizinleri (varsayılan: ev dizininiz)
search_dirs_linux = ["/home/kullanici"]

# Windows için tarama dizinleri
search_dirs_windows = ["C:\\Users\\kullanici", "D:\\Projeler"]

# Hariç tutulacak klasör adları (gizli olanlar zaten atlanır)
exclude_dirs = ["target", "node_modules", "build", "venv", "cache"]

# İndeks dosyasının konumu
index_file = "/home/kullanici/.nav/.nav_index.json"

# Minimum eşleşme skoru (0 = kapalı). Skor kabaca harf başına ~20 artar.
fuzzy_threshold = 0

# Gösterilecek sonuç sayısı (3-100)
limit = 10

# Her açılışta otomatik tarama
auto_scan = false
```

| Ayar | Açıklama |
|---|---|
| `search_dirs_linux` / `search_dirs_windows` | Taranacak kök dizinler. İç içe verilen dizinler (örn. `/home` ve `/home/user`) bir kez taranır. |
| `exclude_dirs` | Adı bu listedeki bir değerle tam eşleşen klasörler atlanır. |
| `fuzzy_threshold` | Bu skorun altındaki sonuçlar gösterilmez. Gürültülü sonuçlar çok geliyorsa artırın. |
| `limit` | Listede gösterilecek en fazla sonuç sayısı. |
| `auto_scan` | `true` ise her açılışta yeniden tarama yapılır. Büyük dizinlerde yavaş olabilir, bu yüzden genelde `nav --scan` tercih edilir. |

> 💡 Yeni klasörler oluşturduğunuzda `n --scan` ile indeksi güncelleyin.

## Yeni sürüm yayınlama

`Cargo.toml` içindeki `version` değerini artırın, ardından etiketi gönderin:

```bash
git tag v0.2.0 && git push origin v0.2.0
```

GitHub Actions; Linux (x86_64, ARM64), macOS (Intel, Apple Silicon) ve Windows binary'lerini derleyip Releases sayfasına yükler. Kurulum betikleri her zaman en son sürümü indirir.

## Geliştirme

```bash
cargo test
```

```bash
cargo clippy --all-targets
```

Proje yapısı:

```
src/
├── main.rs      # argümanlar, indeks yükleme/kaydetme
├── settings.rs  # ~/.nav/Settings.toml okuma
├── scanner.rs   # paralel dizin tarama
├── ui.rs        # ratatui arayüzü ve fuzzy arama
└── help.rs      # --help metni
```

## Lisans

[MIT](LICENSE)

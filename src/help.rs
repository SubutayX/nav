use std::io::IsTerminal;

// stderr'e yazılır: `n` shell fonksiyonu stdout'u yakalayıp `cd` yapar,
// yardım metni stdout'a gitseydi `n --help` metne cd yapmaya çalışırdı.
pub fn print_help() {
    let color = std::io::stderr().is_terminal();
    let (b, c, d, r) = if color {
        ("\x1b[1m", "\x1b[1;36m", "\x1b[2m", "\x1b[0m")
    } else {
        ("", "", "", "")
    };
    let version = env!("CARGO_PKG_VERSION");

    eprintln!(
        "\
{c}nav{r} {version} {d}— fuzzy arama ile hızlı dosya/klasör navigasyonu{r}

{b}KULLANIM{r}
    n [SEÇENEK]          {d}shell fonksiyonu: seçilen klasöre cd yapar{r}
    nav [SEÇENEK]        {d}seçilen yolu sadece ekrana yazar{r}

{b}SEÇENEKLER{r}
    {c}--scan{r}               Dizinleri yeniden tarar ve indeksi günceller
    {c}-h{r}, {c}--help{r}           Bu yardım metnini gösterir
    {c}-V{r}, {c}--version{r}        Sürümü gösterir

{b}KISAYOLLAR{r}
    {c}yazmak{r}               Fuzzy arama ({d}\"calisma\" → Çalışma{r})
    {c}↑ ↓{r}                  Sonuçlar arasında gezin
    {c}Enter{r}                Seçilen klasöre git {d}(dosyaysa bulunduğu klasöre){r}
    {c}Tab{r}                  Sadece klasörler / tüm dosyalar
    {c}Esc{r}, {c}Ctrl+C{r}          Çık

{b}DOSYALAR{r}
    ~/.nav/Settings.toml    Ayarlar {d}(taranacak dizinler, hariç tutulanlar…){r}
    ~/.nav/.nav_index.json  İndeks

{d}Yeni klasörler görünmüyorsa: n --scan
Daha fazlası: https://github.com/SubutayX/nav{r}"
    );
}

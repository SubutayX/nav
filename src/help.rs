pub fn print_help() {
    println!(
        r#"
Navigator (nav) - Hızlı Dosya ve Klasör Navigasyon Aracı

KULLANIM:
    nav [SEÇENEKLER]

SEÇENEKLER:
    --scan          İndeks dosyasını siler ve tüm dizinleri yeniden tarar.
    --help, -h      Bu yardım metnini gösterir.

BİLGİ:
    - Hiçbir seçenek girilmezse, mevcut indeksi yükler ve TUI ekranını açar.
    - TUI içinde 'TAB' tuşu ile sadece klasörler veya tüm dosyalar arasında geçiş yapabilirsiniz.
    - Yukarı/Aşağı ok tuşları ile sonuçlar arasında gezinirsiniz.
    - Enter tuşu ile seçilen konuma terminalde 'cd' yaparsınız (shell fonksiyonu gerekir, README'ye bakın).
    - ESC veya Ctrl+C ile çıkarsınız.

KONFİGÜRASYON:
    Ayarlar: ~/.nav/Settings.toml
    İndeks:  ~/.nav/.nav_index.json
"#
    );
}

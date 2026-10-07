#!/bin/sh
# nav kurulum betiği (Linux / macOS)
#   curl -fsSL https://raw.githubusercontent.com/SubutayX/nav/main/install.sh | sh
set -eu

REPO="SubutayX/nav"
INSTALL_DIR="${NAV_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)" in
    Linux) os="unknown-linux-musl" ;;
    Darwin) os="apple-darwin" ;;
    *) echo "Desteklenmeyen işletim sistemi: $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="aarch64" ;;
    *) echo "Desteklenmeyen mimari: $(uname -m)" >&2; exit 1 ;;
esac

url="https://github.com/$REPO/releases/latest/download/nav-$arch-$os.tar.gz"
echo "İndiriliyor: $url"
mkdir -p "$INSTALL_DIR"
curl -fsSL "$url" | tar -xzf - -C "$INSTALL_DIR" nav
chmod +x "$INSTALL_DIR/nav"
echo "✓ nav kuruldu: $INSTALL_DIR/nav"

# Shell entegrasyonu: `n` fonksiyonu seçilen klasöre cd yapar
add_line() { # $1 = dosya, $2 = içerik
    if [ -f "$1" ] && grep -q "nav-shell-init" "$1"; then
        echo "✓ $1 zaten ayarlı"
    else
        mkdir -p "$(dirname "$1")"
        printf '\n%s\n' "$2" >> "$1"
        echo "✓ 'n' fonksiyonu eklendi: $1"
    fi
}

case "$(basename "${SHELL:-sh}")" in
    fish)
        add_line "$HOME/.config/fish/functions/n.fish" \
'# nav-shell-init
function n
    set -l dir (nav $argv)
    and test -n "$dir"
    and cd "$dir"
end' ;;
    zsh)
        add_line "$HOME/.zshrc" \
'n() { local dir; dir="$(nav "$@")" && [ -n "$dir" ] && cd "$dir"; } # nav-shell-init' ;;
    bash)
        add_line "$HOME/.bashrc" \
'n() { local dir; dir="$(nav "$@")" && [ -n "$dir" ] && cd "$dir"; } # nav-shell-init' ;;
    *)
        echo "! Shell tanınmadı, 'n' fonksiyonunu README'ye bakarak elle ekleyin." ;;
esac

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "! $INSTALL_DIR PATH'te değil. Shell ayar dosyanıza ekleyin." ;;
esac

echo
echo "Yeni bir terminal açıp 'n' yazarak başlayın."

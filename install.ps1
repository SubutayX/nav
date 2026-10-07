# nav kurulum betiği (Windows)
#   irm https://raw.githubusercontent.com/SubutayX/nav/main/install.ps1 | iex
$ErrorActionPreference = "Stop"

$repo = "SubutayX/nav"
$installDir = Join-Path $env:LOCALAPPDATA "nav\bin"
$url = "https://github.com/$repo/releases/latest/download/nav-x86_64-pc-windows-msvc.zip"
$zip = Join-Path $env:TEMP "nav.zip"

Write-Host "İndiriliyor: $url"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Invoke-WebRequest -Uri $url -OutFile $zip
Expand-Archive -Path $zip -DestinationPath $installDir -Force
Remove-Item $zip
Write-Host "✓ nav kuruldu: $installDir\nav.exe"

# Kullanıcı PATH'ine ekle
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ";") -notcontains $installDir) {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "✓ PATH'e eklendi"
}

# Shell entegrasyonu: `n` fonksiyonu seçilen klasöre cd yapar
if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Force -Path $PROFILE | Out-Null }
if (-not (Select-String -Path $PROFILE -Pattern "nav-shell-init" -Quiet)) {
    Add-Content -Path $PROFILE -Value "`nfunction n { `$dir = nav @args; if (`$dir) { Set-Location `$dir } } # nav-shell-init"
    Write-Host "✓ 'n' fonksiyonu eklendi: $PROFILE"
}

Write-Host "`nYeni bir terminal açıp 'n' yazarak başlayın."

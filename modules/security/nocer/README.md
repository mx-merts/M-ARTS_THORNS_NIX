# NOCER — NixOS Runtime Certificate Manager

> **NO**S + **CER**tificate. Okul/kurumsal ağ sertifikalarını (MEB gibi) terminal üzerinden anlık aç/kapa yapmak için yazılmış hafif bir Rust aracıdır.

## Neden?

NixOS'ta sistem CA deposu **read-only**'dir (`/etc/ssl/certs/ca-certificates.crt` → nix store). Yani bir sertifikayı `security.pki.certificateFiles` ile eklemek `nixos-rebuild switch` gerektirir. Okul ağına girip çıkarken her seferinde rebuild yapmak istemiyorsan, NOCER tam sana göre.

## Ne Yapar?

- İndirdiğin `.crt` / `.pem` / `.cer` / `.der` dosyasını **otomatik algılar ve gerekirse PEM'e çevirir**
- Sertifikayı **runtime'da aktifleştirir** (rebuild yok, boot yok, ~100ms)
- Aktifken **Firefox, VS Code, curl, git, python, node ve diğer tüm HTTPS araçları** sertifikayı otomatik kullanır
- İşin bitince tek tuşla deaktive edersin, kendi internetine dönersin
- Her şey `wheel` grubu izniyle çalışır, `sudo` gerekmez

## Kurulum

```nix
# hosts/portable/default.nix içine
imports = [
  ../../modules/security/nocer
];

programs.nocer.enable = true;

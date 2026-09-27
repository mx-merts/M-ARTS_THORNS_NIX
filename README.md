# NixOS Tasinabilir Sistem - Modul Haritasi

## Nereye ne yazilir?

- Yeni program/paket kurmak            -> modules/programs/base.nix
- Donanim sorunu (Wifi, ses, GPU)      -> modules/hardware/ altinda ilgili dosya
- Bluetooth ayari                      -> modules/hardware/bluetooth.nix
- USB/kamera/otomatik baglama          -> modules/hardware/peripherals.nix
- Firewall / port acma                 -> modules/security/firewall.nix
- KDE ozel ayar                        -> modules/desktop/kde.nix
- Yeni arka plan servisi               -> modules/services/ altina yeni dosya
- Makineye ozel ayar                   -> hosts/portable/default.nix icinde specialisation
- Kullanici ayarlari                   -> modules/users/m-arts.nix
- Bootloader / kernel                  -> modules/core/boot.nix
- Ag / hostname                        -> modules/core/networking.nix
- DNS / VPN / gizlilik ayari            -> modules/network/ altinda ilgili dosya
- Saat dilimi / dil / klavye           -> modules/core/locale.nix

## Klasor yapisi

hosts/portable/         bu makinenin profili + donanim taramasi
modules/core/           sistemin iskeleti (boot, nix, locale, networking)
modules/network/         DNS, VPN, gizlilik/hiz ayarlari (dnscrypt-proxy, wireguard)
modules/hardware/       fiziksel donanim (firmware, ses, bluetooth, cevre birimleri)
modules/hardware/gpu/   GPU'ya ozel, makineye kilitli ayarlar (PRIME busId gibi)
modules/security/       firewall, sertlestirme
modules/desktop/        masaustu ortamlari (KDE, ileride Hyprland)
modules/services/       arka plan servisleri (yazici, ileride docker vs.)
modules/programs/       kullanici paketleri, amac bazli
modules/users/          kullanici tanimlari

## Specialisation'lar (boot menusunden secilir)

DESK  - Varsayilan. NVIDIA'siz her makinede guvenli. Fallback (nouveau/modesetting).
PRIME - Sadece bu laptop (RTX 4050 Max-Q + Raptor Lake-P). PRIME offload, sabit busId.

## Yeni bir host eklemek

hosts/<yeni-isim>/ altina default.nix + hardware-configuration.nix ekle,
flake.nix'e nixosConfigurations.<yeni-isim> olarak bagla.

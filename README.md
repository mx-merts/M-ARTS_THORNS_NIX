# MARTS THORNS

## Nereye ne yazılır?

- Yeni program/paket kurmak            -> modules/programs/base.nix
- Uygulama (oyun/gelistirme/ofis)      -> modules/apps/ altinda ilgili dosya
- Flatpak uygulamasi ekle              -> modules/services/flatpak.nix
- Terminal araci (cli)                 -> modules/programs/cli-tools.nix
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
- DNS / VPN / gizlilik ayari           -> modules/network/ altinda ilgili dosya
- Saat dilimi / dil / klavye           -> modules/core/locale.nix

## Klasor yapisi

hosts/portable/         bu makinenin profili + donanim taramasi
modules/core/           sistemin iskeleti (boot, nix, locale, networking)
modules/network/        DNS, VPN, gizlilik/hiz ayarlari (dnscrypt-proxy, proton-vpn)
modules/hardware/       fiziksel donanim (firmware, ses, bluetooth, cevre birimleri)
modules/hardware/gpu/   GPU'ya ozel, makineye kilitli ayarlar (PRIME busId gibi)
modules/security/       firewall, sertlestirme, AppArmor, Firejail, ClamAV+Lynis , NOCER
modules/desktop/        masaustu ortamlari (KDE, ileride Hyprland)
modules/services/       arka plan servisleri (yazici, flatpak)
modules/programs/       kullanici paketleri, amac bazli
modules/apps/           gelistirme, oyun, ofis, tasarim, gunluk uygulamalar, minecraft
modules/users/          kullanici tanimlari

## Specialisation'lar (boot menusunden secilir)

DESK            - Varsayilan (ana config, specialisation degil). NVIDIA'siz her
                  makinede guvenli, Intel/AMD iGPU fallback (nouveau/modesetting).
PRIME           - Sadece bu laptop (RTX 4050 Max-Q + Raptor Lake-P). Intel iGPU
                  varsayilan + NVIDIA offload. busId sabit, makineye kilitli.
                  nvidia.open = true (Ada Lovelace, Turing+).
NVIDIA-DESKTOP  - Tek NVIDIA GPU'lu masaustu senaryosu. busId pinleme yok,
                  agnostik. nvidia.open = false (Maxwell'den Ada'ya kadar
                  tum nesillerde calisir, genis uyumluluk).
                  Gercek NVIDIA masaustu donaniminda HENUZ TEST EDILMEDI.

## Rebuild sayaci ve GRUB isimlendirme

rebuild.sh her calistiginda .rebuild-count dosyasindaki sayiyi 1 arttirir,
NIXOS_LABEL_VERSION=rN olarak ortam degiskenine koyar ve --impure ile
nixos-rebuild'e gecirir. Boylece GRUB menusunde "NixOS - PRIME-r21" gibi
temiz isimler cikar; tarih placeholder'i (1970-01-01) gorunmez.

.rebuild-count git'te TAKIP EDILMEZ (her rebuild'de degisir, gereksiz commit
uretir). Ilk kurulumda dosya yoksa rebuild.sh "0" varsayar, sorun cikmaz.

## Yeni bir host eklemek

hosts/<yeni-isim>/ altina default.nix + hardware-configuration.nix ekle,
flake.nix'e nixosConfigurations.<yeni-isim> olarak bagla.

## Baska diske/makineye tasima

hardware-configuration.nix icindeki dosya sistemi UUID'leri (fileSystems."/"
ve fileSystems."/boot") BU FLASH'A OZELDIR. Yeni bir diske tasindiginda:
  1. nixos-generate-config --root /mnt ile yeni hardware-configuration.nix uret
  2. Eski dosyayi bununla degistir (UUID'ler otomatik dogru gelir)
  3. rebuild.sh ile switch et

Modüller (modules/) hic dokunulmaz - tasima sadece hosts/ seviyesinde olur.

## Deklaratif olmayan katmanlar (henuz)

KDE Plasma gorsel ayarlari (tema, ikon, panel duzeni, kisayollar) ve SDDM
temasi su an ~/.config altinda imperative olarak yasiyor. Home-Manager +
plasma-manager entegrasyonu ile ileride deklaratif hale getirilecek.

## Lisans

Bu proje **CC BY-NC-SA 4.0** (Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International) lisansı altındadır.

- **Atıf (BY):** Kullanan kişi "M-ARTS" adını belirtmek zorundadır.
- **Ticari Değil (NC):** Ticari amaçla kullanılamaz, satılamaz.
- **Aynı Lisansla Paylaş (SA):** Değiştirilip paylaşılırsa aynı lisansla paylaşılmalıdır.

Detaylar için [LICENSE](LICENSE) dosyasına bakın.

Telif hakkı (c) 2026 M-ARTS - https://github.com/mx-merts

## Kisa Komutlar

- **Rebuild:** `sudo /etc/nixos/rebuild.sh`
  Sayaci arttirir, NIXOS_LABEL_VERSION ile switch eder.

- **GitHub'a yedekle:** `sudo /etc/nixos/push.sh` (veya `sudo /etc/nixos/push.sh "commit mesaji"`)
  Tum degisiklikleri add + commit + push eder. Degisiklik yoksa atlar.

## Git Notu

Bu repo'ya root kullanici SSH key ile baglanir (/root/.ssh/id_ed25519).
Bu yuzden git komutlari sudo ile calistirilmalidir.

# NERV-0.0.2 — Hardware Nervous System

> THORNS'un sinir sistemi. Sistemin takıldığı her bilgisayarda donanımı tarar, eksik sürücüleri bağlar, hangi GRUB girdisinin uygun olduğunu söyler ve parçaları terminalden açıp kapatmanı sağlar. SPINE güvenliği izler; NERV donanımı.

## Ne yapar?

| Parça | Ne zaman | Ne yapar |
| --- | --- | --- |
| `nerv-boot` servisi | Her açılışta bir kez (root) | Sürücüsü olduğu hâlde bağlanmamış parçaları bağlar, sonucu `/run/nerv/boot`'a yazar |
| `nerv-notify` servisi | Her girişte bir kez (kullanıcı) | Söylenecek bir şey varsa bildirim gönderir; her şey yolundaysa sessiz kalır |
| `NERV` komutu | Sen çalıştırınca | Bütün parçaları listeler, tek tek yönetir |

**Makine hafızası yoktur.** `/run/nerv` tmpfs'tedir; her açılışta NERV makineyi sıfırdan tanır. Sistem bir sürü farklı bilgisayara takılacağı için bilinçli olarak böyle.

## Kullanım

```bash
NERV                          # bütün rapor
NERV BT                       # tek bölüm
NERV GPU 2 INFO               # bir parçanın ayrıntıları
sudo NERV BT 1 ENABLE         # aç (bağlanmamışsa sürücüsünü bağla)
sudo NERV CAM 1 DISABLE       # kapat
sudo NERV WF 1 RESET          # sürücüyü ayırıp yeniden bağla
sudo NERV CONSOLE             # NERV ▸ istemi; NERV yazmadan komut gir
NERV HELP
```

Komut yapısı her zaman aynı: **`NERV <PARÇA> <N> <EYLEM>`**. `<N>`, listede o bölümdeki sıra numarası. Büyük/küçük harf fark etmez (`nerv bt 1 enable` da çalışır). Okumak herkese açık; değiştirmek `sudo` ister.

### Parçalar

| Uzun | Kısa | | Uzun | Kısa |
| --- | --- | --- | --- | --- |
| `GPU` | | | `TOUCHPAD` | `TP` |
| `AUDIO` | `AUD` | | `KEYBOARD` | `KB` |
| `WIFI` | `WF` | | `DISK` | |
| `ETHERNET` | `ETH` | | `USB` | |
| `BLUETOOTH` | `BT` | | `THUNDERBOLT` | `TB` |
| `CAMERA` | `CAM` | | `BATTERY` | `BAT` |
| `FINGERPRINT` | `FP` | | | |

### Eylemler ve nasıl yapıldıkları

| Parça | ENABLE / DISABLE | RESET |
| --- | --- | --- |
| Wi-Fi, Bluetooth | Radyo anahtarı (rfkill); Bluetooth ayrıca açılır/kapanır | Sürücüyü ayırıp yeniden bağlar |
| Kamera, parmak izi, USB | USB yetkisi (`authorized`): çekirdek aygıtı takılı değilmiş gibi görür | Yetkiyi alıp geri verir (yeniden takmak gibi) |
| Ethernet | PCI: sürücü ayır/bağla; USB: USB yetkisi | Sürücüyü ayırıp yeniden bağlar |
| Dokunmatik yüzey, klavye | Çekirdeğin giriş susturması (`inhibited`) | Sustur ve aç |
| Disk (yalnızca USB) | DISABLE: bölümleri ayırır, sürücüyü kapatır (`udisksctl`). Geri açmak için yeniden tak | — |
| Ses kartı | — | Sürücüyü ayırıp yeniden bağlar (susan sesi kurtarır) |
| Ekran kartı, pil, Thunderbolt | Yalnızca `INFO` | — |

Bağlanmamış (`✖`) bir parçada `ENABLE`, sürücüsünü yüklemeyi ve bağlamayı dener.

### Kurallar

- **Değişiklikler yeniden başlatınca sıfırlanır.** Radyo anahtarları istisna: onları systemd hatırlar, KDE'deki düğme gibi.
- **Sistemi öldürecek şeyleri reddeder:** THORNS'un çalıştığı disk ve ona giden USB yolu, ekran kartları, tek kalan klavye. Dahili diskler kapatılmaz.
- **Politikaya saygı duyar:** `hardening.nix` kara listesindeki sürücüler (FireWire gibi) hiçbir zaman yüklenmez; bu parçalar `⊘ BLOCKED BY POLICY` olarak görünür.
- **Açılışta yalnızca ekler:** eksik sürücüyü bağlar, hiçbir şeyi ayırmaz.

## Rapordaki işaretler

| İşaret | Anlamı |
| --- | --- |
| `●` | Çalışıyor |
| `○` | Bilerek kapalı |
| `◐` | Sürücüsü eksikti, NERV açılışta bağladı |
| `✖` | Görüldü ama bağlanamadı (nedeni `INFO`'da ve raporun altında) |
| `⊘` | Sürücüsü `hardening.nix` tarafından engellendi |

## Ekran kartı önerisi

NERV, NVIDIA kartın kuşağını `pci.ids` veritabanındaki çip adından (veritabanında yoksa PCI kimlik aralığından) bulur:

| Kuşak | Örnek | Önerilen girdi |
| --- | --- | --- |
| NVIDIA yok | Intel, AMD | DESK |
| Kepler ve öncesi | GTX 600/700 | DESK (nouveau) |
| Maxwell, Pascal, Volta | GTX 900/1000 | NVIDIA-L |
| Turing ve sonrası | GTX 16, RTX 20–50 | NVIDIA |
| Turing ve sonrası, bu laptop | RTX 4050 + Intel, adresler `prime-bus.nix` ile aynı | PRIME |

Şu anki girdiyi NERV kendisi bilir: NixOS modülü her girdi için `/etc/nerv/nerv.conf`'a `session=` yazar.

## Giriş bildirimleri

Kendi kanalı var (SPINE'dan bağımsız). Gönderdikleri:

- `NVIDIA PASCAL DETECTED` — `NVIDIA-L RECOMMENDED (current: DESK)`: yanlış girdiyle açıldıysan (ekranda kalır)
- `2 DEVICES SEEN BUT NOT BOUND` — bağlanamayan parçalar (ekranda kalır)
- `1 DEVICE BOUND AT BOOT` — açılışta NERV'in bağladıkları
- `FINGERPRINT READER DETECTED` — parmak izi okuyucu varsa

Ne göndereceğini görmek için: `NERV NOTIFY --DRY`.

## Başlık

Kitty'de başlık **ƝΞᚱꝞ** olarak çizilir: Ɲ (kancalı N), Ξ (Yunan), ᚱ (rün R), Ꝟ (çizgili V). Dört harf de **Junicode** yazı tipinden gelir (`terminal.nix`: `fonts.packages` + kitty `symbol_map`), böylece boyları eşit olur. TTY'de ve boru (`|`) çıktısında düz `NERV` yazılır.

## Dosyalar

| Dosya | Ne |
| --- | --- |
| `default.nix` | NixOS modülü: servisler, `/etc/nerv/nerv.conf` |
| `package.nix` | Rust paketi; çalışma anı araçları (modprobe, bluetoothctl, nmcli, udisksctl, notify-send) ve `pci.ids` |
| `source/src/main.rs` | Komut satırı ve konsol |
| `source/src/scan.rs` | Tarama: başlık bilgileri, PCI/USB listesi, sürücü politikası |
| `source/src/probe.rs` | Her parça türü için ayrı tarayıcı |
| `source/src/gpu.rs` | NVIDIA kuşakları ve girdi önerisi |
| `source/src/action.rs` | ENABLE / DISABLE / RESET ve güvenlik kuralları |
| `source/src/boot.rs` | Açılış taraması |
| `source/src/notify.rs` | Giriş bildirimleri |
| `source/src/report.rs` | Renkler ve bütün çıktı biçimi |
| `source/src/state.rs` | `/run/nerv` dosyaları |
| `source/src/model.rs` | Parça, durum ve aygıt tanımları |
| `source/src/sys.rs` | sysfs okuma, komut çalıştırma, parça adları |

## Sınırlar

- Parmak izi okuyucular yalnızca görünür; THORNS'ta parmak izi desteği (`services.fprintd`) şimdilik kapalı.
- Ekran kartları NERV'den açılıp kapatılmaz; doğru sürücü için GRUB girdisi seçilir.
- Kapatılan USB diski tekrar açmak için çıkarıp takmak gerekir.
- Bilinen sorunlar için otomatik çözüm (`FIX`) sonraki sürümde: önce gerçek vakalar toplanacak.

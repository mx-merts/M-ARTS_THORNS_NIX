# M-ARTS THORNS

**Türkçe** | [English](README.en.md)

Flake tabanlı, modüler ve deklaratif bir NixOS yapılandırması. Ayarlar, konularına göre ayrı modüllere bölünmüştür; bir şeyi değiştirmek istediğinizde nereye bakmanız gerektiğini aşağıdaki tablodan hemen bulabilirsiniz.

## Nereye ne yazılır?

| Konu | Nereye yazılır? |
| --- | --- |
| Yeni program/paket kurulumu | `modules/programs/base.nix` |
| Terminal aracı (CLI) | `modules/programs/cli-tools.nix` |
| Kabuk/terminal ayarları | `modules/programs/shell.nix` |
| Neovim (kurulum ve sistem geneli ayarlar) | `modules/programs/neovim.nix` |
| Neovim kişisel ayarları (eklentiler, kısayollar, tema) | `modules/programs/nvim/` (`~/.config/nvim` buraya bağlanır, `sudo` gerekmez) |
| `.jar` dosyası çalıştırma | `modules/programs/jrun.nix` |
| Yerel (native) Linux uygulaması çalıştırma | `modules/programs/lrun.nix` |
| Uygulama (oyun, geliştirme, ofis) | `modules/apps/` altında ilgili dosya |
| Flatpak uygulaması | `modules/services/flatpak.nix` |
| Yeni arka plan servisi | `modules/services/` altına yeni dosya |
| Donanım sorunu (Wi-Fi, ses, GPU) | `modules/hardware/` altında ilgili dosya |
| Bluetooth ayarı | `modules/hardware/bluetooth.nix` |
| USB / kamera / otomatik bağlama | `modules/hardware/peripherals.nix` |
| DNS / VPN / gizlilik ayarı | `modules/network/` altında ilgili dosya |
| Güvenlik duvarı / port açma | `modules/security/firewall.nix` |
| Sertifika yönetimi | `modules/security/` altında NOCER |
| KDE'ye özel ayarlar | `modules/desktop/kde.nix` |
| Kullanıcı ayarları | `modules/users/m-arts.nix` |
| Önyükleyici / çekirdek | `modules/core/boot.nix` |
| Ağ / makine adı (hostname) | `modules/core/networking.nix` |
| Saat dilimi / dil / klavye | `modules/core/locale.nix` |
| Makineye özel ayar | `hosts/portable/default.nix` içinde `specialisation` |

## Klasör ve dosya yapısı

| Yol | Açıklama |
| --- | --- |
| `flake.nix` | Flake'in giriş noktası; sistem yapılandırmaları (host'lar) burada tanımlanır |
| `flake.lock` | Bağımlılık sürümlerini sabitleyen kilit dosyası |
| `rebuild.sh` | Etkileşimli rebuild betiği |
| `push.sh` | Değişiklikleri GitHub'a yedekleyen betik |
| `hosts/portable/` | Bu makinenin profili ve donanım taraması |
| `modules/core/` | Sistemin iskeleti (`boot`, `nix`, `locale`, `networking`) |
| `modules/network/` | DNS, VPN ve gizlilik ayarları (`dnscrypt-proxy`, `proton-vpn`) |
| `modules/hardware/` | Fiziksel donanım (donanım yazılımı, ses, Bluetooth, çevre birimleri) |
| `modules/hardware/gpu/` | GPU'ya özel, makineye kilitli ayarlar (PRIME `busId` gibi) |
| `modules/security/` | Güvenlik duvarı, sertleştirme, AppArmor, Firejail, ClamAV + Lynis ve NOCER (sertifika yöneticisi) |
| `modules/desktop/` | Masaüstü ortamları (KDE; ileride Hyprland) |
| `modules/services/` | Arka plan servisleri (yazıcı, Flatpak) |
| `modules/programs/` | Amaca göre ayrılmış kullanıcı paketleri (`base`, `shell`, `cli-tools`, `neovim`, `nvim-config`, `jrun`, `lrun`) ve Neovim ayarları (`nvim/`) |
| `modules/apps/` | Geliştirme, oyun, ofis, tasarım, günlük kullanım uygulamaları ve Minecraft |
| `modules/users/` | Kullanıcı tanımları |
| `assets/sddm/pixel-night-city/` | SDDM giriş ekranı teması (`pixel-night-city`) dosyaları |

## Özel modüller

nixpkgs'ta bulunmayan, bu depo için özel olarak geliştirilmiş modüller:

- **NOCER - NixOS sertifika yöneticisi.** `modules/security/` altındadır ve sertifikaların deklaratif olarak yönetilmesini sağlar. Ayrıntılar için kendi README dosyasına bakın.
- **JRUN - `.jar` çalıştırıcı.** `modules/programs/jrun.nix` dosyasındadır. Java `.jar` dosyalarını sorunsuz çalıştırmak için geliştirilmiştir.
- **LRUN - yerel Linux uygulama çalıştırıcı.** `modules/programs/lrun.nix` dosyasındadır. Yerel (native) Linux uygulamalarını sorunsuz çalıştırmak için geliştirilmiştir.

## Specialisation'lar

Specialisation'lar, NixOS'un `specialisation` özelliğiyle oluşturulan alternatif sistem sürümleridir. Önyükleme menüsünden (GRUB) seçilirler.

- **DESK:** Varsayılan seçenektir; ana yapılandırmanın kendisidir, specialisation değildir. NVIDIA ekran kartı olmayan her makinede güvenle çalışır; Intel/AMD tümleşik ekran kartını (iGPU) kullanır (nouveau/modesetting).
- **PRIME:** Yalnızca bu dizüstü bilgisayar içindir (RTX 4050 Max-Q + Raptor Lake-P). Varsayılan olarak Intel iGPU kullanılır; NVIDIA ekran kartı offload yöntemiyle devreye girer. `busId` sabit olduğu için bu sürüm makineye kilitlidir. `hardware.nvidia.open = true` (Ada Lovelace; Turing ve sonrası).
- **NVIDIA-DESKTOP:** Tek NVIDIA ekran kartlı masaüstü senaryosu içindir. `busId` sabitlenmez, yani makineden bağımsızdır. `hardware.nvidia.open = false` olarak ayarlanmıştır ve geniş uyumluluk için Maxwell'den Ada'ya kadar tüm nesillerde çalışacak şekilde hazırlanmıştır. **Gerçek bir NVIDIA masaüstü donanımında henüz test edilmedi.**

## Kısa komutlar

Aşağıdaki komutlar, deponun `/etc/nixos` altında bulunduğunu varsayar.

```bash
# Yeniden derleme ve uygulama (etkileşimli)
sudo /etc/nixos/rebuild.sh

# Değişiklikleri GitHub'a yedekleme
sudo /etc/nixos/push.sh

# Aynı işlem, kendi commit mesajınızla
sudo /etc/nixos/push.sh "commit mesajı"
```

- **`rebuild.sh`** etkileşimli çalışır ve her aşamada onay ister (ayrıntılar aşağıda). Sayaç yalnızca `switch` başarıyla tamamlandığında artar.
- **`push.sh`** tüm değişiklikleri ekler (`add`), commit'ler ve GitHub'a gönderir (`push`). Henüz gönderilmemiş commit'ler varsa onları da gönderir. Yapılacak bir şey yoksa işlemi atlar.

> **Not:** Bu depoya `root` kullanıcısı SSH anahtarıyla (`/root/.ssh/id_ed25519`) bağlanır. Bu yüzden git komutları `sudo` ile çalıştırılmalıdır.

> **Not:** Sadece Neovim ayarlarını (`modules/programs/nvim/`) değiştirdiysen rebuild gerekmez, `sudo /etc/nixos/push.sh` yeterli. Rebuild yalnızca sistem paketleri (`neovim.nix`) için gerekir.

## Rebuild akışı

`rebuild.sh` etkileşimli çalışır ve her aşamada onay ister:

1. **`git add -A`:** Tüm değişiklikler (yeni dosyalar dahil) git'e eklenir. Flake'ler yalnızca git'in bildiği dosyaları görür; bu adım sayesinde yeni bir modül dosyası eklediğinizde `path does not exist` hatası almazsınız.
2. **Durum ve onay:** Git durumu gösterilir ve devam etmek için onay istenir.
3. **`dry-activate`:** Yapılandırma tamamen derlenir ve bir değişiklik raporu verilir, ancak **hiçbir şey uygulanmaz**. Bu aşama başarısız olursa `switch` çalışmaz ve `push` yapılmaz. Ardından onay istenir.
4. **`switch`:** Yeni yapılandırma sisteme uygulanır. Ardından onay istenir.
5. **`push`:** Değişiklikler GitHub'a gönderilir ve bir özet gösterilir.

### Önemli notlar

- Sayaç (`.rebuild-count`) yalnızca `switch` başarıyla tamamlandığında artar. Başarısız veya yarıda bırakılan denemeler numara harcamaz.
- GRUB girdileri, `NIXOS_LABEL_VERSION` sayesinde `T-61` gibi sade adlar alır. `nixos-rebuild`'in varsayılan tarih yer tutucusu (`1970-01-01`) görünmez.
- Tüm onay sorularında yalnızca Enter'a basmak "hayır" anlamına gelir (güvenli varsayılan).
- `switch` başarısız olursa `push` atlanır; böylece bozuk bir yapılandırma GitHub'a yedeklenmez.
- `push` sırasında GitHub tarafında geçici bir hata olursa (örneğin `fatal error in commit_refs`) commit'ler yerelde güvende kalır. `push.sh` bir sonraki çalıştırmada bekleyen commit'leri kendiliğinden gönderir.
- `.rebuild-count` git tarafından **takip edilmez**; her başarılı rebuild'de değiştiği için gereksiz commit'lere yol açardı. İlk kurulumda bu dosya yoksa `rebuild.sh` sayacı `0` kabul eder.

## Yeni bir host eklemek

1. `hosts/<isim>/` klasörünü oluşturun ve içine `default.nix` ile `hardware-configuration.nix` dosyalarını ekleyin.
2. `flake.nix` içine bu host'u `nixosConfigurations.<isim>` olarak ekleyin.

## Başka bir diske veya makineye taşıma

`hardware-configuration.nix` içindeki dosya sistemi UUID'leri (`fileSystems."/"` ve `fileSystems."/boot"`) bu flash belleğe özeldir. Sistemi başka bir diske taşırken şunları yapın:

1. `nixos-generate-config --root /mnt` komutuyla yeni bir `hardware-configuration.nix` üretin.
2. Eski dosyayı bununla değiştirin (UUID'ler otomatik olarak doğru gelir).
3. `rebuild.sh` ile yeni yapılandırmayı uygulayın.

`modules/` klasörüne hiç dokunmanız gerekmez; taşıma yalnızca `hosts/` düzeyinde yapılır.

## Henüz deklaratif olmayan kısımlar

Aşağıdakiler şu an `~/.config` altında elle yönetiliyor:

- KDE Plasma görsel ayarları (tema, simgeler, panel düzeni, kısayollar)


İleride Home Manager ve plasma-manager entegrasyonuyla bunlar da deklaratif olarak yönetilecek.

## Lisans

Bu proje **CC BY-NC-SA 4.0** (Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International) lisansı altındadır.

- **Atıf (BY):** Projeyi kullananlar "M-ARTS" adını belirtmek zorundadır.
- **Ticari Olmayan (NC):** Proje ticari amaçla kullanılamaz ve satılamaz.
- **Aynı Lisansla Paylaş (SA):** Değiştirilip paylaşılırsa aynı lisansla paylaşılmalıdır.

Ayrıntılar için [LICENSE](LICENSE) dosyasına bakın.

Telif hakkı © 2026 M-ARTS - <https://github.com/mx-merts>

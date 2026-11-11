# M-ARTS THORNS

Flake tabanlı, modüler ve deklaratif bir NixOS yapılandırması. **Taşınabilir bir SSD'den** açılacak şekilde tasarlanmıştır; evde, okulda ya da başka bir bilgisayarda aynı sistem açılır. Ayarlar konularına göre ayrı modüllere bölünmüştür; bir şeyi değiştirmek istediğinizde nereye bakmanız gerektiğini aşağıdaki tablodan hemen bulabilirsiniz.

## Nereye ne yazılır?

| Konu | Nereye yazılır? |
| --- | --- |
| Yeni program/paket kurulumu | `modules/programs/base.nix` |
| Terminal aracı (CLI) | `modules/programs/cli-tools.nix` |
| Kabuk/terminal ayarları | `modules/programs/shell.nix`, `modules/desktop/terminal.nix` (kitty, starship) |
| Neovim (kurulum ve sistem geneli ayarlar) | `modules/programs/neovim.nix` |
| Neovim kişisel ayarları (eklentiler, kısayollar, tema) | `modules/programs/nvim/` (`~/.config/nvim` buraya bağlanır, `sudo` gerekmez) |
| `.jar` dosyası çalıştırma | `modules/programs/jrun.nix` |
| Yerel (native) Linux uygulaması çalıştırma | `modules/programs/lrun.nix` |
| Uygulama (oyun, geliştirme, ofis) | `modules/apps/` altında ilgili dosya |
| Flatpak uygulaması ve sandbox izinleri | `modules/services/flatpak.nix` |
| Yeni arka plan servisi | `modules/services/` altına yeni dosya |
| Donanım sorunu (Wi-Fi, ses, GPU) | `modules/hardware/` altında ilgili dosya |
| NVIDIA ayarı (PRIME / masaüstü) | `modules/hardware/gpu/` |
| Bluetooth ayarı | `modules/hardware/bluetooth.nix` |
| USB / kamera / otomatik bağlama | `modules/hardware/peripherals.nix` |
| Casper Excalibur (fan, klavye RGB) | `modules/hardware/excalibur-wmi.nix`, `excalibur-rgb.nix` |
| DNS / gizlilik ayarı | `modules/network/dns.nix`, `privacy.nix` |
| VPN (Proton) / Cloudflare WARP | `modules/network/vpn.nix`, `warp.nix` |
| Güvenlik duvarı / port açma | `modules/security/firewall.nix` |
| Kernel sertleştirme (sysctl, modül kara listesi) | `modules/security/hardening.nix` |
| Virüs tarama (ClamAV), yanlış alarm listesi | `modules/security/scanning.nix` |
| Firefox politikaları | `modules/security/firefox.nix` |
| Sertifika yönetimi | `modules/security/nocer/` (NOCER) |
| KDE'ye özel ayarlar | `modules/desktop/kde.nix` |
| Kullanıcı ayarları | `modules/users/m-arts.nix` |
| Önyükleyici / çekirdek | `modules/core/boot.nix` |
| Otomatik haftalık güncelleme | `modules/core/auto-upgrade.nix` |
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
| `modules/core/` | Sistemin iskeleti (`boot`, `nix-settings`, `locale`, `networking`, `auto-upgrade`) |
| `modules/network/` | DNS (`dnscrypt-proxy`), Proton VPN, Cloudflare WARP, gizlilik ve TCP ayarları |
| `modules/hardware/` | Fiziksel donanım (ses, Bluetooth, çevre birimleri, sensörler, Excalibur sürücüsü) |
| `modules/hardware/gpu/` | GPU'ya özel ayarlar (PRIME `busId` gibi) |
| `modules/security/` | Güvenlik katmanları (aşağıdaki "Güvenlik" bölümüne bakın) |
| `modules/desktop/` | Masaüstü (KDE, Hyprland + Serpantinum), SDDM teması, terminal |
| `modules/services/` | Arka plan servisleri (yazıcı, Flatpak) |
| `modules/programs/` | Amaca göre ayrılmış kullanıcı paketleri (`base`, `shell`, `cli-tools`, `neovim`, `nvim-config`, `jrun`, `lrun`, `cscan`) ve Neovim ayarları (`nvim/`) |
| `modules/apps/` | Geliştirme, oyun, ofis, tasarım ve günlük kullanım uygulamaları |
| `modules/users/` | Kullanıcı tanımları |
| `assets/sddm/pixel-night-city/` | SDDM giriş ekranı teması (`pixel-night-city`) dosyaları |

## Özel modüller

nixpkgs'ta bulunmayan, bu depo için özel olarak geliştirilmiş modüller:

- **SPINE - güvenlik olay toplayıcı ve bildirim servisi (Rust).** `modules/security/spine/` altındadır. LKRG, AIDE, auditd ve ClamAV'dan gelen olayları toplar ve masaüstü bildirimine çevirir. Hiçbir şeyi engellemez, sadece haber verir. `warn` ve `crit` seviyesindeki bildirimler sen kapatana kadar ekranda kalır.
- **NOCER - NixOS sertifika yöneticisi (Rust).** `modules/security/nocer/` altındadır. Okul (MEB) gibi kurumların ağ sertifikalarını rebuild yapmadan açıp kapatır; Firefox politikalarını da buna göre günceller. Ayrıntılar için [kendi README dosyasına](modules/security/nocer/README.md) bakın.
- **thorns-aide - dosya bütünlüğü denetimi.** `modules/security/aide/` altındadır. AIDE'yi sadece `/nix/store` ve git'in kapsamadığı yerlerde çalıştırır: `/boot` ve oturum açılışında çalışan "kalıcılık noktaları".
- **CSCAN - elle virüs tarama.** `modules/programs/cscan.nix` dosyasındadır. ClamAV ile `/`, `/home`, `/boot` ya da verilen yolu paralel olarak tarar.
- **JRUN - `.jar` çalıştırıcı.** `modules/programs/jrun.nix` dosyasındadır. Java `.jar` dosyalarını sorunsuz çalıştırmak için geliştirilmiştir.
- **LRUN - yerel Linux uygulama çalıştırıcı.** `modules/programs/lrun.nix` dosyasındadır. Yerel (native) Linux uygulamalarını sorunsuz çalıştırmak için geliştirilmiştir.

## Güvenlik

**Felsefe:** azami koruma, ama günlük kullanıma dokunmadan. Program kurarken, oyun oynarken ya da internette gezinirken araya soru ya da engel girmez. Bunun yerine sistem şüpheli bir şey gördüğünde **SPINE üzerinden bildirim gönderir**; tıpkı Windows Defender gibi "kur ve unut" mantığıyla çalışır. Bu yüzden katmanların çoğu engelleme değil, **tespit ve bildirim** modundadır.

| Katman | Ne yapar? | Dosya |
| --- | --- | --- |
| Kernel sertleştirme | Kernel adreslerini gizler; ptrace, BPF, userfaultfd ve perf'i kısıtlar; belleği kullanımda sıfırlar (`init_on_alloc`); eski ağ protokollerini, dosya sistemlerini ve FireWire'ı kapatır | `hardening.nix` |
| LKRG | Çalışan kernel'in ve süreç yetkilerinin bütünlüğünü izler; yetki yükseltme girişimlerini yakalar. **Sadece log modunda** çalışır | `lkrg.nix` |
| AIDE | `/boot` ve kalıcılık noktalarındaki değişiklikleri yakalar | `aide/` |
| auditd | Kritik dosyalara ve modül yüklemelerine kimin dokunduğunu kaydeder | `auditing.nix` |
| ClamAV | Downloads, Desktop ve USB'yi anlık tarar. Haftalık `/home` taraması yapar. fangfrisch ile Sanesecurity imzalarını da ekler | `scanning.nix` |
| AppArmor | Complain modunda: sadece log yazar, hiçbir şeyi engellemez | `apparmor.nix` |
| Oyun sandbox'ı | Lutris ve Heroic Flatpak'te ev dizinine erişemez; sadece oyun klasörlerini görür (`~/OYUNLAR`, `~/GAMES`). Bir oyun ya da hileyle gelen token çalıcı Discord, Firefox ve `~/.ssh` dosyalarını göremez | `flatpak.nix` |
| Firefox | Telemetri kapalı, sadece HTTPS, takip koruması, uBlock Origin, Safe Browsing kilitli, punycode adresler açıkça gösterilir, PDF içindeki JavaScript kapalı | `firefox.nix` |
| Ağ | Şifreli ve zararlı alan adı filtreli DNS (Cloudflare Security + Quad9). LLMNR, mDNS ve DHCP'de hostname gönderimi kapalı. Ping'e cevap verilmez. MAC adresi ağa göre farklı | `network/`, `firewall.nix` |
| sudo-rs | Bellek açısından güvenli, Rust ile yazılmış `sudo` | `hosts/portable/default.nix` |

### Bilerek açık bırakılanlar

- **Disk şifreleme (LUKS) ve USBGuard yok.** USBGuard sistemin kendi SSD'sini kilitliyordu. Disk şifrelemesinin olmaması, taşınabilir bir sistemde bilinen en büyük risk.
- **LRUN, JRUN, terminalden çalıştırılan `wine` ve Steam** sandbox dışında çalışıyor. Bunlarla çalıştırılacak dosyalar önceden taranmalı.
- **Proton VPN** Stealth protokolü için 4.18 sürümünü bekliyor (aşağıya bakın).

### Yanlış alarmlar

Doğrulanmış yanlış alarm imzaları `scanning.nix` içindeki `.ign2` listesine **gerekçesi ve tarihiyle birlikte** eklenir. Bir imzanın yanlış alarm olduğuna karar vermeden önce dosyanın SHA-256 özeti VirusTotal'da kontrol edilir; Steam dosyalarında Steam'in "dosya bütünlüğünü doğrula" seçeneği de kullanılabilir.

### Güvenlik komutları

```bash
spine                          # SPINE durumu
spine log                      # Son güvenlik olayları
sudo thorns-aide check all     # AIDE kontrolünü elle çalıştır
sudo thorns-aide accept persist  # Kalıcılık noktalarındaki BİLİNEN bir değişikliği onayla
sudo CSCAN -HOME               # /home'u elle tara
```

## Ağ: DNS, WARP ve Proton

- **DNS:** Tek DNS otoritesi `dnscrypt-proxy` (`dns.nix`). Sorgular şifreli gider ve zararlı alan adları filtrelenir.
- **Cloudflare WARP (`warp.nix`):** **Her açılışta kendiliğinden bağlanır.** Modem, okul ağı ve internet sağlayıcı hangi sitelere girildiğini göremez; sağlayıcı yasakları aşılır. `tunnel_only` modunda çalıştığı için DNS'e dokunmaz; DNS yine `dnscrypt-proxy`'den geçer. Konum Türkiye görünmeye devam eder; WARP bir gizlilik VPN'i değildir. Hız kaybı yaklaşık %15-20 (16.3 → 13.5 Mbit/s ölçüldü).
  - Geçici kapatmak için: `warp-cli disconnect` (bir sonraki açılışta yine bağlanır)
  - Durum: `warp-cli status`
- **Proton VPN (`vpn.nix`):** Ülke seçimi ve gerçek gizlilik için. nixpkgs'teki sürüm 4.16.5. Engellere karşı Stealth protokolü 4.18 ile geldi; nixpkgs'e girmesi `proton-vpn-api-core` kütüphanesinin güncellenmesini bekliyor. **WARP ile aynı anda açılmaz:** önce `warp-cli disconnect`.

## Specialisation'lar

Specialisation'lar, NixOS'un `specialisation` özelliğiyle oluşturulan alternatif sistem sürümleridir. Önyükleme menüsünden (GRUB) seçilirler.

- **DESK:** Varsayılan seçenektir; ana yapılandırmanın kendisidir, specialisation değildir. NVIDIA ekran kartı olmayan her makinede güvenle çalışır; Intel/AMD tümleşik ekran kartını (iGPU) kullanır (nouveau/modesetting).
- **PRIME:** Yalnızca bu dizüstü bilgisayar içindir (RTX 4050 Max-Q + Raptor Lake-P). Varsayılan olarak Intel iGPU kullanılır; NVIDIA ekran kartı offload yöntemiyle devreye girer. `busId` sabit olduğu için bu sürüm makineye kilitlidir. `hardware.nvidia.open = true` (Ada Lovelace; Turing ve sonrası). Flatpak oyunlarının Vulkan'ı çalışabilsin diye `nvidia_uvm` modülü açılışta yüklenir.
- **NVIDIA-DESKTOP:** Tek NVIDIA ekran kartlı masaüstü senaryosu içindir. `busId` sabitlenmez, yani makineden bağımsızdır. `hardware.nvidia.open = false` olarak ayarlanmıştır ve geniş uyumluluk için Maxwell'den Ada'ya kadar tüm nesillerde çalışacak şekilde hazırlanmıştır. **Gerçek bir NVIDIA masaüstü donanımında henüz test edilmedi.**

> **Uyarı:** `nixos-rebuild switch --specialisation ...` **kullanmayın.** Bu komut GRUB'u specialisation'ın kendi yapılandırmasından kurar ve menüdeki diğer girdileri siler. `rebuild.sh` bunun yerine GRUB'u her zaman ana yapılandırmadan kurar (`boot`), seçilen specialisation'ı ise çalışan oturuma `test` ile uygular.

## Kısa komutlar

Aşağıdaki komutlar, deponun `/etc/nixos` altında bulunduğunu varsayar.

```bash
# Yeniden derleme ve uygulama (etkileşimli)
sudo /etc/nixos/rebuild.sh

# Değişiklikleri GitHub'a yedekleme
sudo /etc/nixos/push.sh

# Aynı işlem, kendi commit mesajınızla
sudo /etc/nixos/push.sh "commit mesajı"

# Sisteme kurulu bir paketin sürümü (flake.lock'taki nixpkgs'e göre)
sudo nix eval --impure --raw /etc/nixos#nixosConfigurations.portable.pkgs.proton-vpn.version; echo
```

- **`rebuild.sh`** etkileşimli çalışır ve her aşamada onay ister (ayrıntılar aşağıda).
- **`push.sh`** tüm değişiklikleri ekler (`add`), commit'ler ve GitHub'a gönderir (`push`). Henüz gönderilmemiş commit'ler varsa onları da gönderir. Yapılacak bir şey yoksa işlemi atlar.

> **Not:** Bu depo GitHub'a, sadece bu repoya erişimi olan bir **deploy key** ile (`/root/.ssh/`) bağlanır. Bu yüzden git komutları `sudo` ile çalıştırılmalıdır. Anahtarın sadece yorumunu değiştirmek için `ssh-keygen -c -C "yeni yorum" -f <dosya>` kullanın; anahtarı yeniden üretirseniz yeni anahtarı GitHub'a da eklemeniz gerekir.

> **Not:** Sadece Neovim ayarlarını (`modules/programs/nvim/`) değiştirdiysen rebuild gerekmez, `sudo /etc/nixos/push.sh` yeterli. Rebuild yalnızca sistem paketleri (`neovim.nix`) için gerekir.

## Rebuild akışı

`rebuild.sh` etkileşimli çalışır ve her aşamada onay ister. Tüm onay sorularında yalnızca Enter'a basmak "hayır" anlamına gelir (güvenli varsayılan).

1. **Git hazırlığı:** Tüm değişiklikler (yeni dosyalar dahil) `git add -A` ile eklenir. Flake'ler yalnızca git'in bildiği dosyaları görür; bu sayede yeni bir modül dosyası eklediğinizde `path does not exist` hatası almazsınız. Git durumu gösterilir ve onay istenir. Ardından rebuild kilidi alınır (haftalık otomatik güncellemeyle çakışmasın diye) ve AIDE `/boot`'u kontrol eder.
2. **Güncelleme (isteğe bağlı):** `y` derseniz bütün flake girdileri (`nix flake update`) ve Flatpak uygulamaları güncellenir. Enter'a basarsanız mevcut sürümlerle devam edilir. Güncelleme kurulmazsa (derleme başarısız olursa, işlemi iptal ederseniz ya da Ctrl+C'ye basarsanız) `flake.lock` kendiliğinden eski haline döner; bozuk bir güncelleme repoda kalmaz.
3. **Dry rebuild (`dry-activate`):** Yapılandırma tamamen derlenir ve bir değişiklik raporu verilir, ancak **hiçbir şey uygulanmaz**. Bu aşama başarısız olursa rebuild yapılmaz ve `push` atlanır.
4. **Rebuild:** Bir hedef menüsü çıkar: DESK, NVIDIA-DESKTOP, PRIME (Enter = şu an açık olan) ya da "sadece GRUB" (bir sonraki açılışta uygulanır). GRUB her zaman ana yapılandırmadan kurulur; seçilen hedef çalışan oturuma uygulanır. Ardından AIDE `/boot` için yeni bir referans kaydı alır.
5. **Push:** Onay verilirse değişiklikler GitHub'a gönderilir.
6. **Özet:** Her adımın sonucu bir tabloda gösterilir.

### Önemli notlar

- Sayaç (`.rebuild-count`) yalnızca yeni nesil GRUB'a kurulduğunda artar. Başarısız veya yarıda bırakılan denemeler numara harcamaz.
- GRUB girdileri, `NIXOS_LABEL_VERSION` sayesinde `T-113` gibi sade adlar alır. `nixos-rebuild`'in varsayılan tarih yer tutucusu (`1970-01-01`) görünmez.
- Rebuild başarısız olursa `push` atlanır; böylece bozuk bir yapılandırma GitHub'a yedeklenmez.
- `push` sırasında GitHub tarafında geçici bir hata olursa (örneğin `fatal error in commit_refs`) commit'ler yerelde güvende kalır. `push.sh` bir sonraki çalıştırmada bekleyen commit'leri kendiliğinden gönderir.
- `.rebuild-count` git tarafından **takip edilmez**; her başarılı rebuild'de değiştiği için gereksiz commit'lere yol açardı. İlk kurulumda bu dosya yoksa `rebuild.sh` sayacı `0` kabul eder.

## Otomatik bakım

| Ne | Ne zaman | Not |
| --- | --- | --- |
| `thorns-upgrade` (nixpkgs güncellemesi) | Pazar 04:00 | Yeni sistemi sadece bir sonraki açılış için hazırlar, çalışan oturuma dokunmaz. Bilgisayar o saatte kapalıysa açılınca çalışır |
| fangfrisch (ek ClamAV imzaları) | Saatlik | |
| AIDE kontrolü | Günlük | |
| ClamAV `/home` taraması | Haftalık (pazartesi) | Bilgisayar kapalıysa açıldıktan birkaç dakika sonra çalışır; sadece log tutar |
| Lynis denetimi | Haftalık | Sadece log tutar |
| Nix store bütünlük kontrolü | Aylık | |

## Bilinen tuzaklar ve çözümleri

- **LKRG ve modül yükleme yarışı:** LKRG, `systemd-modules-load` ile aynı anda yüklenince o anda yüklenmekte olan diğer modüller (`nvidia_uvm`, `kvm_intel`) `Interrupted system call` hatasıyla düşüyordu. Bu yüzden LKRG ayrı bir servisle (`lkrg-load`) diğer modüllerden **sonra** yüklenir.
- **Flatpak oyunları NVIDIA'da açılmıyor (`vr -3`):** `nvidia_uvm` yüklü değilse Vulkan aygıtı oluşturulamaz. Flatpak bu modülü kendisi yükleyemediği için modül açılışta yüklenir (`nvidia-laptop.nix`).
- **Flatpak'te HTTPS indirmeleri takılıyor:** `SSL_CERT_FILE` değişkeni `/run/nocer-ca-bundle.crt` dosyasını gösteriyor, ama `/run` Flatpak'in içinden görünmüyor. Bu yüzden dosya oyun başlatıcılarına salt okunur olarak açılır (`flatpak.nix`).
- **Flatpak izinleri ancak uygulama yeniden başlayınca geçerli olur:** önce `flatpak kill <uygulama>`. Kullanıcı seviyesindeki geçici izinleri temizlemek için `flatpak override --user --reset <uygulama>`.

## Yeni bir host eklemek

1. `hosts/<isim>/` klasörünü oluşturun ve içine `default.nix` ile `hardware-configuration.nix` dosyalarını ekleyin.
2. `flake.nix` içine bu host'u `nixosConfigurations.<isim>` olarak ekleyin.

## Başka bir diske veya makineye taşıma

`hardware-configuration.nix` içindeki dosya sistemi UUID'leri (`fileSystems."/"` ve `fileSystems."/boot"`) bu SSD'ye özeldir. Sistemi başka bir diske taşırken şunları yapın:

1. `nixos-generate-config --root /mnt` komutuyla yeni bir `hardware-configuration.nix` üretin.
2. Eski dosyayı bununla değiştirin (UUID'ler otomatik olarak doğru gelir).
3. `rebuild.sh` ile yeni yapılandırmayı uygulayın.

`modules/` klasörüne hiç dokunmanız gerekmez; taşıma yalnızca `hosts/` düzeyinde yapılır. WARP kaydı ve modu yeni kurulumda kendiliğinden yapılır.

## Henüz deklaratif olmayan kısımlar

Aşağıdakiler şu an `~/.config` altında elle yönetiliyor:

- KDE Plasma görsel ayarları (tema, simgeler, panel düzeni, kısayollar)
- Hyprland kullanıcı ayarları (`~/.config/hypr/hyprland.conf`; ilk rebuild'de bir başlangıç dosyası kopyalanır)

İleride Home Manager ve plasma-manager entegrasyonuyla bunlar da deklaratif olarak yönetilecek.

## Lisans

Bu proje **CC BY-NC-SA 4.0** (Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International) lisansı altındadır.

- **Atıf (BY):** Projeyi kullananlar "M-ARTS" adını belirtmek zorundadır.
- **Ticari Olmayan (NC):** Proje ticari amaçla kullanılamaz ve satılamaz.
- **Aynı Lisansla Paylaş (SA):** Değiştirilip paylaşılırsa aynı lisansla paylaşılmalıdır.

Ayrıntılar için [LICENSE](LICENSE) dosyasına bakın.

Telif hakkı © 2026 M-ARTS - <https://github.com/mx-merts>

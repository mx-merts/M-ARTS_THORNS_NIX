# SKIN-0.0.2 — Network Membrane

> THORNS'un derisi: dış dünyayla aradaki zar. Ağ güvenliğinin bütün katmanlarını tek komutla sıkar ya da gevşetir. SPINE güvenliği izler, NERV donanımı; SKIN ağı.

## Ne yapar?

| Parça | Ne zaman | Ne yapar |
| --- | --- | --- |
| `skin-watch` servisi | Her açılışta, ağdan önce (root) | KILLSWITCH ve LAN kurallarını durumla eşit tutar, SEVER/ROT zamanlayıcısını işletir, tüneli ve DNS'i izler |
| `skin-notify` servisi | Giriş boyunca (kullanıcı) | Bekçinin olaylarını bildirime çevirir |
| `SKIN` komutu | Sen çalıştırınca | Durumu gösterir, katmanları değiştirir |

**Her açılış SEAL başlar.** Durum `/run/skin`'de (tmpfs); yeniden başlatınca bütün değişiklikler silinir. Kalıcı olan tek şey `SKIN TRUST` listesi.

## Üç durum

| Durum | İşaret | Ne zaman | Süre |
| --- | --- | --- | --- |
| **SEAL** | ● | Her zaman. En güçlü hâli | Kalıcı |
| **SEVER** | ✂ | Kafe, otel, okul: giriş sayfası (captive portal) olan ağ | 10 dk sonra kendiliğinden SEAL |
| **ROT** | ☠ | Son çare: hiçbir şey çalışmıyorsa | 30 dk sonra kendiliğinden SEAL, önce `ROT` yazdırarak onay ister |

Bütün katmanlar aynı durumdaysa adı **SEALED / SEVERED / ROTTEN**, karışıksa **◆ CUSTOM**. Bir katman istendiği gibi çalışmıyorsa **✖**.

## Kullanım

```bash
SKIN                       # bütün katmanların durumu
SKIN TUN                   # tek katmanın ayrıntısı
sudo SKIN SEVER            # giriş sayfası için aç
sudo SKIN SEAL             # geri kapat
sudo SKIN ROT              # her korumayı bırak (onay ister)
sudo SKIN ROT KS           # sadece kill switch'i kapat
sudo SKIN SEAL KS          # sadece kill switch'i aç
sudo SKIN SEVER TUN DNS    # birden çok katman
sudo SKIN TRUST            # bu Wi-Fi ev: LAN açık kalsın
sudo SKIN DISTRUST         # artık değil
SKIN HELP
```

Komut yapısı: **`SKIN <DURUM> [KATMAN..]`**. Katman verilmezse bütün deri değişir (zamanlayıcılı); verilirse sadece o katmanlar (zamanlayıcısız, sen geri alana kadar kalır). Uzun/kısa ad, büyük/küçük harf fark etmez; kısa adlar (`TUN`, `KS`, `ID`) sadece komut yazarken kullanılır, ekranda ve bildirimlerde hep tam ad görünür. Okumak herkese açık; değiştirmek `sudo` ister.

## Katmanlar

| Katman | Kısa | SEAL | SEVER | ROT |
| --- | --- | --- | --- | --- |
| `DNS` | `DNS` | Şifreli + zararlı alan adı filtreli (dnscrypt-proxy) | Ağın kendi DNS'i | Ağın kendi DNS'i |
| `TUNNEL` | `TUN` | WARP bağlı, bütün trafik içinden | WARP kapalı | WARP kapalı |
| `KILLSWITCH` | `KS` | Tünelin dışına hiçbir şey çıkmaz | Kapalı | Kapalı |
| `LAN` | `LAN` | Bilinmeyen ağda sadece modem; bilinen ağda açık | Sadece modem | Açık |
| `IDENTITY` | `ID` | MAC ağa ve açılışa göre farklı, bilgisayar adı gönderilmez | Değişmez | Gerçek MAC + bilgisayar adı |

### Nasıl yapıldıkları

- **DNS:** SEAL `dnscrypt-proxy`'yi başlatır ve bağlantının DNS ayarını geri yükler (`nmcli device reapply`). SEVER/ROT onu durdurur ve sorguları ağın DNS'ine verir (`resolvectl domain <arayüz> ~.`). Giriş sayfaları ancak böyle açılır: sayfanın adresini sadece ağın DNS'i bilir.
- **TUNNEL:** `warp-cli connect / disconnect`. Bağlanması en fazla 15 sn beklenir.
- **KILLSWITCH ve LAN:** tek bir nftables tablosu, `inet skin`. Her değişiklikte bütünüyle, tek işlemde yeniden kurulur. Sadece fiziksel kartlardan (Wi-Fi, Ethernet) **çıkan** trafiğe bakar; tünel, loopback ve gelen trafik ona dokunmaz (gelen bağlantıları NixOS güvenlik duvarı her durumda kapalı tutar). Kill switch açıkken kartlardan sadece şunlar çıkabilir: adres alma (DHCP, IPv6 komşu keşfi), WARP'ın kendi uç noktaları, şifreli DNS sunucuları ve Proton'un WireGuard uç noktası. Gerisi reddedilir ve sayılır (`SKIN KS` → HELD).
- **IDENTITY:** MAC, NetworkManager'da **bellekte ve geçici olarak** değişir (`nmcli connection modify --temporary`), diske hiçbir şey yazılmaz. Değişiklik Wi-Fi'ı yeniden bağlar. Normal hâli `privacy.nix`'te: MAC her ağda farklı ve her açılışta yeni; aynı açılış içinde kopup bağlanınca aynı kalır (kafe girişi kaybolmasın diye).

## Kafede: giriş sayfası

```bash
sudo SKIN SEVER     # ✂ SEVERED ── neverssl.com'u aç, giriş yap
sudo SKIN SEAL      # ● SEALED ── was open for 2:13
```

Sayfa kendiliğinden açılmazsa tarayıcıda `http://neverssl.com`. SEAL'ı unutursan 10 dk sonra kendisi kapanır. SEVER sırasında kill switch kapalıdır: giriş yapana kadar başka uygulamalar da tünelsiz çıkabilir; kısa tut.

**WARP bu ağda çalışmıyorsa** (bazı okul ve kurum ağları engeller) SEAL `✖ TUNNEL SEVERED` der ve kill switch her şeyi tutar. Tünelsiz devam etmek için tünel ve kill switch birlikte bırakılır; DNS şifreli, LAN izole, MAC gizli kalır:

```bash
sudo SKIN SEVER TUN KS
```

## Bilinen ağlar (ev)

Bilinen ağda SEAL iken LAN açık kalır (yazıcı, telefon). Evde bir kez:

```bash
sudo SKIN TRUST
```

Liste `/var/lib/skin/known`'da, **sadece bu makinede** tutulur. Bilinçli olarak git'e yazılmaz: bir ev SSID'i, Wi-Fi haritalarında (WiGLE gibi) evin yerini bulmaya yeter. Herkese açık olmayan bir depo için Nix seçeneği de var: `programs.skin.knownNetworks = [ "..." ];`.

## Bildirimler

Kendi kanalı var (SPINE ve NERV'den bağımsız). Gönderdikleri:

- `SEALED ── 5 of 5 layers` · `SEVERED ── <ağ>` · `ROTTEN ── <ağ>` — bütün deri değişince
- `KILLSWITCH ROTTED`, `DNS TUNNEL SEVERED` — tek katman değişince
- `TUNNEL SEVERED` — WARP 15 sn'dir yok (ekranda kalır); `TUNNEL ROTTING` — 5 dk'dır yok; `TUNNEL SEALED` — geri geldi
- `DNS SEVERED` / `DNS SEALED` — şifreli DNS cevap vermiyor / yine veriyor
- `KILLSWITCH SEVERED` / `KILLSWITCH SEALED` — kurallar yüklenemedi / yine yüklendi
- `SEVER EXPIRED ── SEALED` — zamanlayıcı doldu, deri kendiliğinden kapandı
- `UNKNOWN NETWORK ── <ağ>` — bilinmeyen bir Wi-Fi'a bağlandın

Ne göndereceğini görmek için: `SKIN NOTIFY --DRY` (açık kalır, Ctrl+C ile çık).

## Acil çıkış

Kill switch yüzünden internet yoksa ve SKIN de yardım etmiyorsa:

```bash
sudo SKIN ROT KS                     # önce bu
sudo nft delete table inet skin      # o da olmazsa: kuralları tamamen sil
```

İkincisinden sonra bekçi kuralları 3 sn içinde geri koyar; kalıcı çözüm için `SKIN ROT KS`. Yeniden başlatmak her şeyi SEAL'a döndürür.

## Proton VPN ile

TUNNEL katmanı WARP'tır. Proton kullanmak için:

```bash
sudo SKIN SEVER TUN KS    # WARP kapanır, kill switch Proton'un girişine izin versin
# Proton'u bağla
sudo SKIN SEAL KS         # kill switch Proton'un uç noktasını tanır ve kapanır
```

Proton bağlıyken bekçi WARP yok diye uyarmaz (TUN zaten SEVER).

## Başlık

Kitty'de başlık **ΣꝀᛁƝ** olarak çizilir: Σ (Yunan), Ꝁ (çizgili K), ᛁ (rün I), Ɲ (kancalı N, italik). NERV'deki ƝΞᚱꝞ gibi dördü de **Junicode**'dan gelir (`terminal.nix`). TTY'de ve boru (`|`) çıktısında düz `SKIN` yazılır.

## Dosyalar

| Dosya | Ne |
| --- | --- |
| `default.nix` | NixOS modülü: servisler, `/etc/skin/skin.conf`, `programs.skin.knownNetworks` |
| `package.nix` | Rust paketi; çalışma anı araçları (nft, nmcli, ip, warp-cli, wg, resolvectl, notify-send) |
| `source/src/main.rs` | Komut satırı ve durum değişikliği |
| `source/src/layers.rs` | Her katmanın SEAL / SEVER / ROT adımları ve sırası |
| `source/src/firewall.rs` | `inet skin` tablosu: KILLSWITCH ve LAN kuralları |
| `source/src/net.rs` | Ağın şu anki hâli: kart, SSID, modem, WARP, MAC |
| `source/src/watch.rs` | Bekçi (`skin-watch`) |
| `source/src/notify.rs` | Bildirimler (`skin-notify`) |
| `source/src/report.rs` | Durum ekranı, katman ayrıntısı, yardım |
| `source/src/paint.rs` | Renkler, başlık, adım satırları |
| `source/src/state.rs` | `/run/skin` ve `/var/lib/skin` dosyaları |
| `source/src/model.rs` | Katman ve durum tanımları |
| `source/src/sys.rs` | Dosya okuma, süre sınırlı komut çalıştırma |

## Sınırlar (0.0.1)

- SEVER'de kill switch tamamen kapalı. "Sadece giriş penceresi çıksın, diğer uygulamalar beklesin" (ayrı Firefox profili + nftables cgroup kuralı) sonraki sürümde.
- TUNNEL katmanı sadece WARP'ı yönetir; Proton elle bağlanır (yukarıda).
- Bilinen ağ sadece Wi-Fi adıyla (SSID) tanınır; Ethernet her zaman "bilinmeyen".
- Bilinen ağda da MAC her açılışta yenidir.

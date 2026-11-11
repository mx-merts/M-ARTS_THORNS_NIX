# NOCER-0.0.2 — NixOS Runtime Certificate Manager

> **NO**S + **CER**tificate. Okul/kurumsal ağ sertifikalarını (MEB gibi) terminal üzerinden anlık aç/kapa yapmak için yazılmış hafif bir Rust aracıdır.

## Neden?

NixOS'ta sistem CA deposu **read-only**'dir (`/etc/ssl/certs/ca-certificates.crt` → nix store). Yani bir sertifikayı `security.pki.certificateFiles` ile eklemek `nixos-rebuild switch` gerektirir. Okul ağına girip çıkarken her seferinde rebuild yapmak istemiyorsan, NOCER tam sana göre.

## Ne yapar?

- İndirdiğin `.crt` / `.pem` / `.cer` / `.der` dosyasını **otomatik algılar ve gerekirse PEM'e çevirir**
- Sertifikayı **runtime'da açar/kapatır** (rebuild yok, boot yok)
- Listeleme (`nocer list`) herkese açık; ekleme/açma/kapama/silme `sudo nocer` ister (state yalnızca root'a yazılabilir)

Açık bir sertifikayı kullananlar:

| Nerede | Kimler | Nasıl |
| --- | --- | --- |
| `/run/nocer-ca-bundle.crt` | curl, git, python, node, nix ve `SSL_CERT_FILE` okuyan her şey | Ortam değişkenleri (anında) |
| Firefox | Firefox | Politika (`Certificates.Install`); Firefox'u yeniden başlatınca |
| `~/.pki/nssdb` | Electron/Chromium uygulamaları: Discord, VS Code, Spotify, Obsidian | `certutil` ile eklenir (depo varsa) |
| Flatpak oyun başlatıcıları | Lutris, Heroic | Bundle salt okunur bağlanır (`flatpak.nix`) |

### Kapatınca gerçekten kapanır

Firefox, politikayla eklenen sertifikayı profilinin kendi deposuna **kalıcı olarak kopyalar** ve listeden çıkınca silmez. Bu yüzden NOCER bir sertifikayı kapatırken ya da silerken onu **parmak iziyle** şu depolardan da temizler:

- Firefox profilleri (`~/.mozilla/firefox/*` ve `~/.config/mozilla/firefox/*`)
- `~/.pki/nssdb` ve `~/.local/share/pki/nssdb`

İşlem `sudo` altında bile **senin kullanıcı kimliğinle** yapılır, dosya sahipliği bozulmaz. Firefox açıksa NOCER yeniden başlatmanı söyler.

## Kullanım

```bash
nocer                         # TUI (okuma); değişiklik için: sudo nocer
nocer list                    # sertifikaları listele
nocer --version               # NOCER-0.0.2

sudo nocer add ~/Downloads/MEB_SERTIFIKASI.cer
sudo nocer enable MEB_SERTIFIKASI.cer
sudo nocer disable MEB_SERTIFIKASI.cer
sudo nocer remove MEB_SERTIFIKASI.cer
```

TUI tuşları: `↑/↓` gezin, `Enter` aç/kapa, `A` ekle (fzf ile seç), `D` sil, `Q` çık.

## Kurulum

```nix
# hosts/portable/default.nix içine
imports = [
  ../../modules/security/nocer
];

programs.nocer.enable = true;
```

Seçenekler (`modules/security/nocer/default.nix`):

| Seçenek | Varsayılan | Açıklama |
| --- | --- | --- |
| `stateDir` | `/var/lib/nocer` | Sertifikalar ve `index.json` |
| `bundlePath` | `/run/nocer-ca-bundle.crt` | Birleşik CA dosyası |
| `setGlobalEnv` | `true` | `SSL_CERT_FILE` ve benzerlerini global ayarlar |
| `manageFirefoxPolicy` | `true` | Firefox politikasını Nix politikaları + sertifikalarla birleştirir |

## Sınırlar

- Ortam değişkenleri yeni açılan programlarda geçerlidir; açık bir program bundle dosyasını her bağlantıda yeniden okumuyorsa yeniden başlatılmalıdır.
- Firefox değişikliği ancak yeniden başlatınca görür.
- `~/.pki/nssdb` hiç oluşmamışsa (Electron uygulaması hiç açılmamışsa) oraya eklenmez.

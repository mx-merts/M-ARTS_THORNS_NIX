# AMAC: GRUB temasi: Elegant (vinceliuice/Elegant-grub2-themes, GPL-3.0).
# Secilen hal: sharp / left / light, 1080p; fotograf: assets/grub/thorns.png
# (THORNS: dikenler arasinda beyaz gul), seritte gul ortalanir.
#
# Upstream'in install.sh / generate.sh'i kullanilmiyor: logoyu "system"
# secenegiyle lsb_release'ten okuyor (Nix sandbox'inda yok) ve kendi flake'i
# ayri bir nixpkgs getiriyor. Dosyalar burada kopyalanir (core.sh ile ayni).
#
# Tema 1920x1080 icin cizilmis: once 1080p denenir, olmazsa "auto".
# Menu adlari ve simge siniflari: modules/core/boot-menu.nix.
#
# Upstream'den farklar:
#   - Fontlar: GRUB fontlari kenar yumusatmasiz (1 bit) cizilir; ince
#     cizgili fontlar kirik gorunur. Menu: Geist Mono SemiBold 22 (Vercel,
#     autohint). E / C ekranlari: Terminus 20 (piksel piksel cizilmis konsol
#     fontu; GRUB'in duzenleyici cercevesi kesiksiz birlesir). Ikisi de sabit
#     genislikli: menu hizasi korunur. Eksik karakter -> NixOS'un unicode.pf2'si.
#   - E / C paneli: theme.txt terminal_box_*.png istiyor ama repoda yok;
#     yazi arka planin ustune basiliyor, okunmuyordu. grub-terminal-box.py
#     yuvarlak koseli, yari saydam, ince lavanta cizgili bir kutu uretir.
#     "Buzlu cam": GRUB terminalin icini saydam yapamaz ama splashImage'i
#     (background_image) ekrana hizali basar; bu yuzden splashImage, arka
#     planin bulaniklastirilip koyu renge boyanmis kopyasidir (frost.png).
#   - Fotograf: Elegant'in dag fotografi yerine kendi resmimiz. sharp / left
#     yerlesiminde fotograf soldaki 862x1080 seritte; resim 1920x1080'i
#     kaplayacak sekilde olceklenir, serit "photoOffset" px sagdan baslar.
#   - Secim cubugu: Elegant'in renkli cubugu "accent" rengine boyanir
#     (siyah-beyaz fotograf icin grafit).
#   - Logo: panel gorselindeki "NiiCE" silinir; yerine grub-brand.svg
#     (M-ARTS, Montserrat ExtraBold harfleri yola cevrilmis) konur.
#     Koordinatlar sharp / left yerlesimine gore (bkz. assertions).
#   - GRUB animasyon desteklemez (gfxmenu statik); gecis efekti yok.

{ config, lib, pkgs, elegant-grub, ... }:

let
  cfg = config.thorns.grubTheme;
  inherit (lib) mkOption types;

  # Renkler: Catppuccin Mocha (kitty ile ayni palet)
  base = "1e1e2e";      # panel zemini
  lavender = "b4befe";  # panel cizgisi

  theme = pkgs.runCommand "grub-theme-elegant-${cfg.background}-${cfg.layout}-${cfg.side}-${cfg.color}" {
    nativeBuildInputs = [ pkgs.grub2 pkgs.python3 pkgs.imagemagick pkgs.librsvg ];
  } ''
    src=${elegant-grub}
    other=$src/assets/assets-other/other-1080p
    mkdir -p $out

    # --- Fontlar ---
    geist=$(find ${pkgs.geist-font} -name GeistMono-SemiBold.otf | head -n 1)
    [ -n "$geist" ] || { echo "grub-theme: GeistMono-SemiBold.otf bulunamadi"; exit 1; }
    term=$(find ${pkgs.terminus_font} -name 'ter-u20n.otb' | head -n 1)
    [ -n "$term" ] || term=$(find ${pkgs.terminus_font} -name 'ter-u20n.pcf*' | head -n 1)
    [ -n "$term" ] || { echo "grub-theme: Terminus 20 (ter-u20n) bulunamadi"; exit 1; }
    grub-mkfont -a -n "Thorns Mono" -s 22 -o $out/thorns-mono-22.pf2 "$geist"
    grub-mkfont -n "Thorns Term" -s 20 -o $out/thorns-term-20.pf2 "$term"

    # --- Tema dosyalari ---
    bg=$src/backgrounds/backgrounds-${cfg.background}/background-${cfg.background}-${cfg.layout}-${cfg.side}-${cfg.color}.jpg
    ${if cfg.photo == null then ''
      cp $bg $out/background.jpg
    '' else ''
      # Fotograf: 1920x1080'i kaplayacak sekilde olcekle, seridi kes, sola yapistir
      magick $bg \( ${cfg.photo} -resize 1920x1080^ -gravity center -extent 1920x1080 +repage \
        -gravity NorthWest -crop 862x1080+${toString cfg.photoOffset}+0 +repage \) \
        -geometry +0+0 -composite -quality 92 $out/background.jpg
    ''}
    cp -r $src/assets/assets-icons-${cfg.color}/icons-${cfg.color}-1080p $out/icons
    cp $src/config/theme-${cfg.layout}-${cfg.side}-${cfg.color}-1080p.txt $out/theme.txt
    chmod u+w $out/theme.txt
    for p in c e w; do
      ${if cfg.accent == null then ''
        cp $other/select_$p-${cfg.background}-${cfg.color}.png $out/select_$p.png
      '' else ''
        magick $other/select_$p-${cfg.background}-${cfg.color}.png -fill '#${cfg.accent}' -colorize 100% PNG32:$out/select_$p.png
      ''}
    done
    cp $other/${cfg.logo}.png $out/logo.png

    # --- E / C paneli: kutu + buzlu cam ---
    python3 ${./grub-terminal-box.py} $out ${base}d6 ${lavender}b0 16 1.5
    magick $out/background.jpg -blur 0x18 -fill '#${base}' -colorize 84% $out/frost.png

    sed -i \
      -e 's|"Unifont Regular 16"|"Thorns Mono Regular 22"|g' \
      -e 's|^terminal-font: .*|terminal-font: "Thorns Term Regular 20"|' \
      -e 's|^terminal-left: .*|terminal-left: "10%"|' \
      -e 's|^terminal-top: .*|terminal-top: "10%"|' \
      -e 's|^terminal-width: .*|terminal-width: "80%"|' \
      -e 's|^terminal-height: .*|terminal-height: "80%"|' \
      -e 's|^terminal-border: .*|terminal-border: "24"|' \
      $out/theme.txt

    # --- Bilgi paneli ve logo ---
    # forest arka planinin kendi bilgi paneli var (core.sh ile ayni kural)
    info=$other/${cfg.layout}-${cfg.side}${lib.optionalString (cfg.background == "forest") "-alt"}.png
    ${if cfg.brand == null then ''
      cp $info $out/info.png
    '' else ''
      # "NiiCE" (sol ust, 150x34 @ 79,86) silinir, yerine logo
      magick $info \( -size 160x44 xc:white \) -geometry +74+81 -compose DstOut -composite PNG32:$out/info.png
      rsvg-convert ${cfg.brand} -o $out/brand.png
      printf '\n+ image {\n  top = 86\n  left = 79\n  file = "brand.png"\n}\n' >> $out/theme.txt
    ''}
  '';
in
{
  options.thorns.grubTheme = {
    background = mkOption {
      type = types.enum [ "forest" "mojave" "mountain" "wave" ];
      default = "mountain";
      description = "Temanin arka plan fotografi.";
    };
    # "blur" disarida: upstream onu farkli dosyalarla kuruyor, kullanmiyoruz.
    layout = mkOption {
      type = types.enum [ "window" "float" "sharp" ];
      default = "sharp";
      description = "Fotograf ve menu yerlesimi.";
    };
    side = mkOption {
      type = types.enum [ "left" "right" ];
      default = "left";
      description = "Fotograf hangi tarafta.";
    };
    color = mkOption {
      type = types.enum [ "light" "dark" ];
      default = "light";
      description = "Menu paneli rengi.";
    };
    photo = mkOption {
      type = types.nullOr types.path;
      default = ../../assets/grub/thorns.png;
      description = ''
        Elegant'in fotografi yerine konacak resim (16:9 iyi olur). Sadece
        sharp / left yerlesiminde (bkz. assertions). null: Elegant'in fotografi.
      '';
    };
    photoOffset = mkOption {
      type = types.ints.between 0 1058;
      default = 155;
      description = "Fotograf seridinin, 1920x1080'e olceklenmis resimde soldan kac px'ten basladigi.";
    };
    accent = mkOption {
      type = types.nullOr (types.strMatching "[0-9a-fA-F]{6}");
      default = "2a2a33";
      description = "Secim cubugunun rengi (RRGGBB). null: Elegant'in kendi rengi.";
    };
    logo = mkOption {
      type = types.str;
      default = "Empty";
      example = "Default";
      description = ''
        Fotografin ustundeki logo: Elegant'in assets/assets-other/other-1080p
        klasorundeki bir dosyanin adi (.png olmadan), or. "Nixos". "Empty":
        logo yok (gul fotografin odak noktasi, ustune logo binmesin).
      '';
    };
    brand = mkOption {
      type = types.nullOr types.path;
      default = ./grub-brand.svg;
      description = ''
        Sol ustteki "NiiCE" yazisinin yerine konacak SVG (225x34 civari).
        null: Elegant'in kendi paneli aynen kalir.
      '';
    };
  };

  config = {
    assertions = [
      {
        assertion = cfg.brand == null || (cfg.layout == "sharp" && cfg.side == "left");
        message = "thorns.grubTheme.brand sadece sharp / left yerlesiminde calisiyor (NiiCE'nin koordinatlari); brand = null yap.";
      }
      {
        assertion = cfg.photo == null || (cfg.layout == "sharp" && cfg.side == "left");
        message = "thorns.grubTheme.photo sadece sharp / left yerlesiminde calisiyor (862x1080 serit); photo = null yap.";
      }
    ];

    boot.loader.grub = {
      inherit theme;
      gfxmodeEfi = "1920x1080,auto";
      # Buzlu cam: E / C panelinin ici bu resimle doldurulur (ust aciklama).
      # Menunun arka plani yine temadan gelir.
      splashImage = "${theme}/frost.png";
      # NixOS splash ile beyaz yazi ayarliyor; biraz daha yumusak gri.
      # (grub.cfg'de tema ve fontlardan sonra calisir)
      extraConfig = ''
        set color_normal=light-gray/black
      '';
    };
  };
}

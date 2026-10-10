{ config, lib, pkgs, ... }:
let
  gtkSchemaPaths = [
    "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}"
    "${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}"
  ];

  lrun = pkgs.writeShellApplication {
    name = "LRUN";
    runtimeInputs = [ pkgs.steam-run pkgs.coreutils ];
    text = ''
      if [ $# -lt 1 ]; then
        echo "usage: LRUN <binary> [args...]" >&2
        exit 1
      fi

      BIN="$1"
      shift

      # Göreli yolu mutlak yola çevir
      case "$BIN" in
        /*) ;;
        *) BIN="$(realpath "$BIN")" ;;
      esac

      if [ ! -f "$BIN" ]; then
        echo "LRUN: not a file: $BIN" >&2
        exit 1
      fi

      # Çalıştırma izni yoksa ekle
      chmod +x "$BIN" 2>/dev/null || true

      # GTK şemaları + mevcut XDG_DATA_DIRS
      export XDG_DATA_DIRS="${lib.concatStringsSep ":" gtkSchemaPaths}:''${XDG_DATA_DIRS:-}"

      # AppImage ise extract-and-run kullan (FUSE olmadan çalışır)
      case "$BIN" in
        *.AppImage|*.appimage)
          cd /tmp
          exec steam-run "$BIN" --appimage-extract-and-run "$@"
          ;;
        *)
          cd /tmp
          exec steam-run "$BIN" "$@"
          ;;
      esac
    '';
  };
in
{
  environment.systemPackages = [ lrun ];
}

{ config, lib, pkgs, ... }:
let
  gtkSchemaPaths = [
    "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}"
    "${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}"
  ];

  jrun = pkgs.writeShellApplication {
    name = "JRUN";
    runtimeInputs = [ pkgs.steam-run pkgs.jdk21 pkgs.coreutils ];
    text = ''
      if [ $# -lt 1 ]; then
        echo "usage: JRUN <file.jar> [args...]" >&2
        exit 1
      fi

      JAR="$1"
      shift

      case "$JAR" in
        /*) ;;
        *) JAR="$(realpath "$JAR")" ;;
      esac

      if [ ! -f "$JAR" ]; then
        echo "JRUN: not a file: $JAR" >&2
        exit 1
      fi

      export XDG_DATA_DIRS="${lib.concatStringsSep ":" gtkSchemaPaths}:''${XDG_DATA_DIRS:-}"

      cd /tmp
      exec steam-run java -jar "$JAR" "$@"
    '';
  };
in
{
  environment.systemPackages = [ jrun ];

  environment.sessionVariables.XDG_DATA_DIRS = gtkSchemaPaths;
}

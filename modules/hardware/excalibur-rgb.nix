{ config, lib, pkgs, ... }:
let
  cfg = config.hardware.excaliburRgb;
  inherit (lib) mkIf mkEnableOption mkOption types;
in
{
  # AMAC: Casper Excalibur klavye RGB ayarini her acilista (ve uykudan
  # donuste) uygular. excalibur-wmi surucusu gerekir. Surucu bagli degilse
  # (baska makine) sessizce hicbir sey yapmaz.
  #
  # NOT: Surucu firmware'den parlaklik okuyamaz ve 0 kabul eder; bu yuzden
  # sira her zaman: once parlaklik, sonra renk, en son efekt.

  options.hardware.excaliburRgb = {
    enable = mkEnableOption "Casper Excalibur klavye RGB ayari";

    mode = mkOption {
      type = types.enum [ "off" "static" "blink" "fade" "heartbeat" "wave" "random" "rainbow" ];
      default = "rainbow";
      description = "Klavye efekti.";
    };

    color = mkOption {
      type = types.strMatching "[0-9A-Fa-f]{6}";
      default = "FFFFFF";
      description = "RRGGBB hex renk (# olmadan). rainbow/random efektlerinde etkisiz.";
    };

    brightness = mkOption {
      type = types.ints.between 0 2;
      default = 2;
      description = "0 = kapali, 1 = orta, 2 = tam.";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.excalibur-rgb = {
      description = "Casper Excalibur klavye RGB ayarini uygula";
      wantedBy = [ "multi-user.target" "post-resume.target" ];
      after = [ "systemd-modules-load.service" "post-resume.target" ];
      serviceConfig.Type = "oneshot";
      path = [ pkgs.coreutils ];
      script = ''
        base=/sys/class/leds/excalibur::kbd_backlight

        # Surucu LED cihazlarini olusturana kadar en fazla 10 sn bekle
        for _ in $(seq 50); do
          [ -e "$base-left/brightness" ] && break
          sleep 0.2
        done
        if [ ! -e "$base-left/brightness" ]; then
          echo "excalibur LED cihazi yok, atlaniyor"
          exit 0
        fi

        echo ${toString cfg.brightness} > "$base-left/brightness"
        echo ${toString cfg.brightness} > "$base-corners/brightness"
        for z in left middle right corners; do
          echo ${cfg.color} > "$base-$z/color"
          echo ${cfg.mode}  > "$base-$z/mode"
        done
      '';
    };
  };
}

{ config, lib, pkgs, ... }:
let
  cfg = config.thorns.bootMenu;
  grub = config.boot.loader.grub;

  # rebuild.sh "T-<sayi>", thorns-upgrade "AUTO-<tarih>" verir, ":-:-:" ile
  # sarili (NIXOS_LABEL_VERSION, --impure gerekir). Elle rebuild: bos.
  rawLabel = builtins.getEnv "NIXOS_LABEL_VERSION";
  bare = lib.removeSuffix ":-:-:" (lib.removePrefix ":-:-:" rawLabel);
  # "T-113" -> "T-[113]", "AUTO-20261115" -> "AUTO-[20261115]", gerisi "DEV"
  m = builtins.match "([A-Z]+)-([0-9]+)" bare;
  tag = if m == null then "DEV" else "${builtins.elemAt m 0}-[${builtins.elemAt m 1}]";

  distro = config.system.nixos.distroName;
  cfgFile = "${(lib.head grub.mirroredBoots).path}/grub/grub.cfg";
  sed = "${pkgs.gnused}/bin/sed";

  # Ad, sabit genislige bosluklarla tamamlanir: GRUB fontu (Unifont) sabit
  # genislikli oldugu icin "·" ve surum butun girdilerde alt alta gelir.
  #   DESK     · T-[113]
  #   NVIDIA-L · T-[113]
  pad = cfg.width - lib.stringLength cfg.name;
  entry = "${cfg.name}${lib.concatStrings (lib.replicate (lib.max 0 pad) " ")} · ${cfg.tag}";
in
{
  # AMAC: GRUB menusunde sade, hizali adlar: "DESK     · T-[113]".
  #
  # nixpkgs'in install-grub.pl'i adlari "<distro> - Default" ve
  # "<distro> - <configurationName>" diye yaziyor; specialisation girdilerine
  # simge sinifi (--class) da vermiyor, temada simgeleri bos kaliyordu.
  # Bu yuzden grub.cfg yazildiktan sonra (extraInstallCommands) duzeltiliyor:
  #   1. ana girdi:            "NixOS - Default"            -> "DESK     · T-[113]"
  #   2. specialisation'lar:   "NixOS - NVIDIA   · T-[113]" -> "NVIDIA   · T-[113]"
  #   3. alt menu:             "NixOS - All configurations" -> "All configurations"
  #   4. sinifsiz her girdi:   --class nixos (NixOS simgesi)
  # "All configurations" icindeki eski nesillere sadece 4. adim dokunur
  # (onlarin adi "NixOS - Configuration 113 - ..." 2. adimin desenine uymaz).
  # Komutlar tekrar calissa da ayni sonucu verir (desenler "NixOS - " arar).

  options.thorns.bootMenu = {
    name = lib.mkOption {
      type = lib.types.strMatching "[A-Z][A-Z0-9-]*";
      default = "DESK";
      description = ''
        Bu yapilandirmanin GRUB'daki adi. Ana yapilandirma "DESK"; her
        specialisation kendi adini verir (hosts/portable/default.nix).
      '';
    };

    width = lib.mkOption {
      type = lib.types.ints.positive;
      default = 8;
      description = ''
        Ad sutununun genisligi: en uzun ad ("NVIDIA-L") kadar. Daha uzun bir
        ad eklenirse bu sayi da buyutulur (yoksa degerlendirme hata verir).
      '';
    };

    label = lib.mkOption {
      type = lib.types.str;
      readOnly = true;
      default = if rawLabel == "" then ":-:-:dev:-:-:" else rawLabel;
      description = "Ham surum etiketi (system.nixos.label icin).";
    };

    tag = lib.mkOption {
      type = lib.types.str;
      readOnly = true;
      default = tag;
      description = ''Menudeki surum: "T-[113]", "AUTO-[20261115]" ya da "DEV".'';
    };
  };

  config = {
    assertions = [{
      assertion = pad >= 0;
      message = "thorns.bootMenu: \"${cfg.name}\" ${toString cfg.width} karakterden uzun; thorns.bootMenu.width'i buyut.";
    }];

    # Specialisation'larin adi buradan okunur (install-grub.pl: configuration-name).
    boot.loader.grub.configurationName = entry;

    # Sadece ana yapilandirmanin komutlari calisir (GRUB hep ondan kurulur).
    boot.loader.grub.extraInstallCommands = ''
      if [ -f ${cfgFile} ]; then
        ${sed} -i \
          -e 's|^menuentry "${distro} - Default" |menuentry "${entry}" |' \
          -e 's|^menuentry "${distro} - \([A-Z][A-Z0-9-]* *· [^"]*\)"|menuentry "\1"|' \
          -e 's|^submenu "${distro} - All configurations"|submenu "All configurations"|' \
          -e 's|^\(menuentry "[^"]*"\) *{$|\1 --class nixos {|' \
          ${cfgFile}
      fi
    '';
  };
}

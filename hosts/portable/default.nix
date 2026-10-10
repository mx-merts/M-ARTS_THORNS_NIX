{ config, lib, pkgs, ... }:
{
  imports = [
    ./hardware-configuration.nix

    ../../modules/core/boot.nix
    ../../modules/core/nix-settings.nix
    ../../modules/core/locale.nix
    ../../modules/core/networking.nix
    ../../modules/core/auto-upgrade.nix

    ../../modules/network/dns.nix
    ../../modules/network/vpn.nix
    ../../modules/network/tuning.nix
    ../../modules/network/privacy.nix

    ../../modules/hardware/agnostic.nix
    ../../modules/hardware/audio.nix
    ../../modules/hardware/bluetooth.nix
    ../../modules/hardware/peripherals.nix

    ../../modules/security/firewall.nix
    ../../modules/security/hardening.nix
    ../../modules/security/apparmor.nix
    ../../modules/security/sandboxing.nix
    ../../modules/security/scanning.nix
    ../../modules/security/nocer

    ../../modules/desktop/kde.nix
    ../../modules/desktop/sddm-theme.nix
    ../../modules/services/printing.nix

    ../../modules/programs/base.nix
    ../../modules/programs/shell.nix
    ../../modules/programs/cli-tools.nix
    ../../modules/programs/jrun.nix
    ../../modules/apps/development.nix
    ../../modules/apps/gaming.nix
    ../../modules/apps/office.nix
    ../../modules/apps/design.nix
    ../../modules/apps/extras.nix
    ../../modules/services/flatpak.nix
    ../../modules/users/m-arts.nix
  ];

  networking.hostName = "THORNS";

  system.stateVersion = "26.05";

  system.nixos.tags = [ "DESK" ];

    system.nixos.label = "NIXOS-DESK-" + (let v = builtins.getEnv "NIXOS_LABEL_VERSION"; in if v == "" then ":-:-:dev:-:-:" else v);

  console.keyMap = "trq";

  environment.sessionVariables.NIXOS_OZONE_WL = "1";

  programs.nocer.enable = true;

  security.sudo.enable = true;
  security.sudo.wheelNeedsPassword = true;

	specialisation."PRIME".configuration = { config, lib, ... }: {
	  boot.loader.grub.configurationName = lib.mkForce ("NIXOS-PRIME-" + (let v = builtins.getEnv "NIXOS_LABEL_VERSION"; in if v == "" then ":-:-:dev:-:-:" else v));
	  imports = [ ../../modules/hardware/gpu/nvidia-laptop.nix ];
	};

  specialisation."NVIDIA-DESKTOP".configuration = { config, lib, ... }: {
    boot.loader.grub.configurationName = lib.mkForce ("NIXOS-NVIDIA-DESKTOP-" + (let v = builtins.getEnv "NIXOS_LABEL_VERSION"; in if v == "" then ":-:-:dev:-:-:" else v));
    imports = [ ../../modules/hardware/gpu/nvidia-desktop.nix ];
  };
}

{ config, lib, pkgs, ... }:
{
  # AMAC: Haftada bir nixpkgs'i gunceller ve yeni sistemi SADECE bir sonraki
  # acilis icin hazirlar ("boot"), calisan oturuma dokunmaz. Bozulursa GRUB'dan
  # onceki nesil secilir. (Resmi autoUpgrade'in --update-input bayragi
  # deprecated oldugu icin kendi servisimizi yaziyoruz.)
  systemd.services.thorns-upgrade = {
    description = "Haftalik otomatik guncelleme (bir sonraki acilis icin)";
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    path = with pkgs; [
      coreutils gnutar xz.bin gzip gitMinimal
      config.nix.package.out config.programs.ssh.package
      config.system.build.nixos-rebuild
    ];
    environment.HOME = "/root";
    script = ''
      set -eu
      cd /etc/nixos
      nix flake update nixpkgs
      nixos-rebuild boot --flake /etc/nixos#portable
    '';
    serviceConfig = {
      Type = "oneshot";
      Nice = 19;
      IOSchedulingClass = "idle";
    };
  };
  systemd.timers.thorns-upgrade = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "Sun 04:00";
      Persistent = true;
      RandomizedDelaySec = "1h";
    };
  };
}

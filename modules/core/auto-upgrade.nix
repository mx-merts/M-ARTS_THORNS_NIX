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
      util-linux
      config.nix.package.out config.programs.ssh.package
      config.system.build.nixos-rebuild
    ];
    environment.HOME = "/root";
    script = ''
      set -eu

      # rebuild.sh ile ayni kilidi paylas: ikisi ayni anda /boot'a veya
      # flake.lock'a dokunamaz. (25 Ekim 2026'da upgrade, rebuild.sh'in
      # switch'i ile cakisti: AIDE yanlis kritik uyari verdi ve flake.lock
      # dogrulanmadan GitHub'a gitti.) /run tmpfs oldugu icin kalinti kilit
      # olmaz; kilit 1 saat alinamazsa bu hafta vazgecilir, Persistent=true
      # sayesinde sonraki hafta tekrar denenir.
      exec 9>/run/thorns-rebuild.lock
      if ! flock -w 3600 9; then
        echo "thorns-upgrade: kilit 1 saat sonra hala mesgul, bu hafta vazgectim."
        exit 1
      fi

      cd /etc/nixos

      # AIDE: check /boot first, so an unattended upgrade never re-baselines a
      # boot partition that was changed outside a rebuild (SPINE gets an alert).
      AIDE=/run/current-system/sw/bin/thorns-aide
      AIDE_CLEAN=0
      if [ -x "$AIDE" ] && "$AIDE" pre-rebuild --alert; then AIDE_CLEAN=1; fi

      nix flake update nixpkgs
      nixos-rebuild boot --flake /etc/nixos#portable

      if [ "$AIDE_CLEAN" = 1 ]; then "$AIDE" post-rebuild; fi
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

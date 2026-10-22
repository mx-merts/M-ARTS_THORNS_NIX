{ config, lib, pkgs, ... }:
{
  # PURPOSE: Kernel-level audit trail. Records "who touched what, and when"
  # for a few high-signal targets. Blocks nothing, changes no behavior.
  # The log (/var/log/audit/audit.log) will be read by BRIAR later.

  security.auditd.enable = true;
  security.audit = {
    enable = true;
    backlogLimit = 8192;
    # NOTE: auditctl -R aborts at the first rule that fails, so every path
    # below must already exist when this service runs. It runs very early in
    # boot (before sysinit.target), so only paths on the root filesystem are
    # safe here. /boot (separate FAT partition) is handled by a separate
    # unit below.
    rules = [
      # Kernel module load/unload. udev-triggered loads (USB, Bluetooth, GPU)
      # will show up too; BRIAR will filter the expected ones.
      "-a always,exit -F arch=b64 -S init_module,finit_module,delete_module -k kernel-modules"

      # Persistence points
      "-w /root/.ssh -p wa -k ssh"
      "-w /var/lib/nocer -p wa -k nocer"
      "-w /home/m-arts/.config/autostart -p wa -k autostart"
      "-w /home/m-arts/.config/systemd/user -p wa -k user-systemd"
    ];
  };

  # Boot chain watches (ESP fallback EFI binary and GRUB files). Loaded only
  # after /boot is mounted. PartOf makes this unit reload whenever the main
  # rules service restarts (it flushes all rules first with -D).
  systemd.services.audit-rules-boot = {
    description = "Load audit watches for the /boot partition";
    wantedBy = [ "multi-user.target" ];
    after = [ "audit-rules-nixos.service" ];
    partOf = [ "audit-rules-nixos.service" ];
    unitConfig.RequiresMountsFor = "/boot";
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = [
        "${pkgs.audit}/bin/auditctl -w /boot/EFI/BOOT -p wa -k boot"
        "${pkgs.audit}/bin/auditctl -w /boot/grub -p wa -k boot"
      ];
    };
  };

  # Make sure the watched root-owned directory exists so its rule can load.
  systemd.tmpfiles.rules = [ "d /root/.ssh 0700 root root -" ];
}

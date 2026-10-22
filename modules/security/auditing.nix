{ config, lib, pkgs, ... }:
{
  # PURPOSE: Kernel-level audit trail. Records "who touched what, and when"
  # for a few high-signal targets. Blocks nothing, changes no behavior.
  # The log (/var/log/audit/audit.log) will be read by BRIAR later.

  security.auditd.enable = true;
  security.audit = {
    enable = true;
    backlogLimit = 8192;
    rules = [
      # Kernel module load/unload. Listed first on purpose: it has no path
      # dependency, so it loads even if a watched path below is missing.
      # Note: udev-triggered loads (USB, Bluetooth, GPU) will show up too;
      # BRIAR will filter the expected ones.
      "-a always,exit -F arch=b64 -S init_module,finit_module,delete_module -k kernel-modules"

      # Boot chain (ESP fallback EFI binary and GRUB files)
      "-w /boot/EFI/BOOT -p wa -k boot"
      "-w /boot/grub -p wa -k boot"

      # Persistence points
      "-w /root/.ssh -p wa -k ssh"
      "-w /var/lib/nocer -p wa -k nocer"
      "-w /home/m-arts/.config/autostart -p wa -k autostart"
      "-w /home/m-arts/.config/systemd/user -p wa -k user-systemd"
    ];
  };

  # Guarantee the watched root directory exists so its rule never fails to load.
  systemd.tmpfiles.rules = [ "d /root/.ssh 0700 root root -" ];
}

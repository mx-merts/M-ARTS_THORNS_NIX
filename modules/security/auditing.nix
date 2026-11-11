{ config, lib, pkgs, ... }:
let
  home = "/home/m-arts";
  auditctl = "${pkgs.audit}/bin/auditctl";

  # Idempotent watch: remove it if already loaded ("-" = ignore "not found"),
  # then add it. Otherwise a restart while the old rules are still loaded
  # fails with "Rule exists".
  watch = path: key: [
    "-${auditctl} -W ${path} -p wa -k ${key}"
    "${auditctl} -w ${path} -p wa -k ${key}"
  ];
in
{
  # PURPOSE: Kernel-level audit trail. Records "who touched what, and when"
  # for a few high-signal targets. Blocks nothing, changes no behavior.
  # The log (/var/log/audit/audit.log) is read by SPINE.

  security.auditd.enable = true;
  security.audit = {
    enable = true;
    backlogLimit = 8192;
    # NOTE: auditctl -R aborts at the first rule that fails, and a watch only
    # loads if its PARENT directory exists. This runs very early in boot
    # (before sysinit.target), so only paths whose parent always exists are
    # here (/root, /var/lib). Everything else is in audit-rules-late below.
    rules = [
      # Kernel module load/unload, for 64-bit AND 32-bit callers (32-bit
      # programs can run here: Steam/Wine need IA32 emulation). udev-triggered
      # loads (USB, Bluetooth, GPU) show up too; SPINE only counts those.
      "-a always,exit -F arch=b64 -S init_module,finit_module,delete_module -k kernel-modules"
      "-a always,exit -F arch=b32 -S init_module,finit_module,delete_module -k kernel-modules"

      # Persistence points
      "-w /root/.ssh -p wa -k ssh"
      "-w /var/lib/nocer -p wa -k nocer"
    ];
  };

  # Watches that need /boot mounted or directories created by tmpfiles
  # (on a fresh install ~/.config/systemd does not exist at first boot).
  # Runs after sysinit (tmpfiles done). PartOf makes it reload whenever the
  # main rules service restarts (that one flushes all rules first with -D).
  systemd.services.audit-rules-late = {
    description = "Load audit watches for /boot and the user's persistence points";
    wantedBy = [ "multi-user.target" ];
    after = [ "audit-rules-nixos.service" ];
    partOf = [ "audit-rules-nixos.service" ];
    unitConfig.RequiresMountsFor = [ "/boot" home ];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart =
        # Boot chain: ESP fallback EFI binary and GRUB files
        watch "/boot/EFI/BOOT" "boot"
        ++ watch "/boot/grub" "boot"
        # Things that run at login
        ++ watch "${home}/.config/autostart" "autostart"
        ++ watch "${home}/.config/systemd/user" "user-systemd";
    };
  };

  # The watched directories must exist for their rules to load.
  systemd.tmpfiles.rules = [
    "d /root/.ssh                       0700 root   root  -"
    "d ${home}/.config/autostart        0755 m-arts users -"
    "d ${home}/.config/systemd          0755 m-arts users -"
    "d ${home}/.config/systemd/user     0755 m-arts users -"
  ];
}

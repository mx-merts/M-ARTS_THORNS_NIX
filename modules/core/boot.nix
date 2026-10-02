{ config, lib, pkgs, ... }:
{
  # AMAC: GRUB, UEFI-only modda. Bu diskte GPT var ama BIOS Boot Partition
  # yok, bu yuzden hybrid (UEFI+BIOS) kurulum yapilamiyor. UEFI-only zaten
  # tema/rice imkanini tam sagliyor, BIOS destegi bu diskte repartition
  # gerektirir - simdilik riske girmiyoruz.
  # efiInstallAsRemovable: NVRAM kaydina bagimli olmadan calisir.

  boot.loader.efi.canTouchEfiVariables = false;

  boot.loader.grub = {
    enable = true;
    efiSupport = true;
    efiInstallAsRemovable = true;
    device = "nodev";   # BIOS/MBR kurulumu YOK, sadece UEFI
    useOSProber = true;
    configurationLimit = 10;
  };

  boot.initrd.systemd.enable = true;
}

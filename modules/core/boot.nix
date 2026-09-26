{ config, lib, pkgs, ... }:
{
  # AMAC: Boot yapisi. GRUB kullaniyoruz - hem gorsel ozellestirme (tema)
  # imkani var hem de hem UEFI hem eski BIOS/MBR makineleri destekliyor.
  # efiInstallAsRemovable: NVRAM kaydina bagimli olmadan, her UEFI
  # makinesinin otomatik denedigi fallback yola kurulum yapar - taşınabilir
  # surucu icin kritik.

  boot.loader.systemd-boot.enable = false;
  boot.loader.efi.canTouchEfiVariables = true;

  boot.loader.grub = {
    enable = true;
    efiSupport = true;
    efiInstallAsRemovable = true;
    device = "/dev/sda";
    useOSProber = true;
  };

  boot.initrd.systemd.enable = true;
}

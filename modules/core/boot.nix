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
    # os-prober KAPALI: rebuild hangi makinede yapilirsa o makinenin
    # disklerini tarayip (okuldaki Windows dahil) girdilerini bu SSD'nin
    # GRUB'ina yaziyordu. Baska bir sistemi acmak icin: firmware'in boot
    # menusu (genelde F12 / F8 / Esc). Windows icin asagidaki WINDOWS girdisi.
    useOSProber = false;
    # WINDOWS: os-prober'in yerine. Rebuild sirasinda degil, HER ACILISTA
    # o anki makinede Windows'un EFI dosyasi aranir; bulunursa girdi
    # gorunur, bulunmazsa (Windows'suz makine) menude hic yer almaz.
    extraEntries = ''
      insmod part_gpt
      insmod fat
      if search --no-floppy --file --set=thorns_win /EFI/Microsoft/Boot/bootmgfw.efi; then
        menuentry "WINDOWS" --class windows {
          insmod chain
          search --no-floppy --file --set=root /EFI/Microsoft/Boot/bootmgfw.efi
          chainloader /EFI/Microsoft/Boot/bootmgfw.efi
        }
      fi
    '';
    configurationLimit = 10;
  };

  # NOT (25 Ekim 2026): "boot.initrd.systemd.enable = true;" satiri kaldirildi.
  # 23 Ekim 2026 nixpkgs'inde (rev 3ec63636...) scripted initrd tamamen
  # kaldirildi; systemd initrd artik tek ve varsayilan yol, secenek reddediliyor.
  # Bu satir thorns-upgrade'i degerlendirme asamasinda kirdi ("no longer has
  # any effect; please remove it"). Davranis degismiyor: zaten true idi.
  boot.tmp.cleanOnBoot = true;  # /tmp her acilista temizlenir
}

{ config, lib, pkgs, ... }:
let
  kernel = config.boot.kernelPackages.kernel;
  kdir = "${kernel.dev}/lib/modules/${kernel.modDirVersion}/build";

  # nixpkgs'in lkrg paketi lkrg.ko'yu elle kopyaliyor ve modul imza sertifikasini
  # (lkrg.ko.cer) uretmiyor. Guncel nixpkgs ise her harici modul icin bu dosyayi
  # bekliyor (kmod/aggregator.nix) -> "cat: extra/lkrg.ko.cer: No such file".
  # Cozum excalibur-wmi.nix ile ayni: kbuild'in kendi modules_install'i modulu
  # imzalar ve .ko.cer'i uretir. (nixpkgs duzeltilince bu override silinebilir.)
  lkrg = config.boot.kernelPackages.lkrg.overrideAttrs (old: {
    installPhase = ''
      runHook preInstall
      make -C ${kdir} M="$PWD" ${lib.escapeShellArgs old.makeFlags} \
        INSTALL_MOD_PATH="$out" INSTALL_MOD_DIR=extra DEPMOD=true \
        modules_install
      runHook postInstall
    '';
  });
in
{
  # AMAC: LKRG (Linux Kernel Runtime Guard, Openwall). Calisan kernel'in
  # butunlugunu ve sureclerin yetki (credential) butunlugunu izler; kernel
  # exploit'lerini (yetki yukseltme girisimlerini) yakalar.
  #
  # SADECE LOG MODU: hicbir seyi engellemez, hicbir sureci oldurmez, kernel'i
  # asla panic'e sokmaz. Tespitler kernel loguna "LKRG: ALERT: ..." olarak
  # yazilir; SPINE bunlari okuyup kritik bildirim gosterir (spine/src/lkrg.rs).
  #
  # Neden modul parametreleri (sysctl profile_enforce degil): profiller sadece
  # sysctl ile ayarlanabiliyor ve sysctl, modul yuklendikten SONRA uygulaniyor.
  # Aradaki kisa surede LKRG varsayilan "strict" modda calisirdi. Parametreler
  # ise modul yuklenirken gecerli, yani en bastan log modunda baslar.
  #
  #   *_enforce=0   : logla ve kabul et (engelleme yok)
  #   umh_validate=0: usermodehelper denetimi kapali. LKRG'nin izin listesi
  #                   FHS yollarini (/sbin/modprobe) bekliyor; NixOS'ta modprobe
  #                   /nix/store'da oldugu icin her USB takilisinda yanlis alarm
  #                   uretirdi. (UMH yine de pCFI ile korunuyor.)
  #
  # Engelleme moduna gecmek icin (birkac hafta sorunsuz calistiktan sonra):
  # once canli test: "sudo sysctl lkrg.profile_enforce=1" (yeniden baslatinca
  # log moduna doner). Sorun yoksa asagidaki *_enforce=0 degerleri kaldirilir.

  boot.extraModulePackages = [ lkrg ];
  # LKRG systemd-modules-load ile ayni anda yuklenince, o an yuklenen diger
  # moduller (nvidia_uvm, kvm_intel...) "Interrupted system call" ile dusuyordu.
  # Bu yuzden diger moduller bittikten SONRA yukleniyor (upstream lkrg.service gibi).
  systemd.services.lkrg-load = {
    description = "Load LKRG after other kernel modules";
    wantedBy = [ "multi-user.target" ];
    after = [ "systemd-modules-load.service" "systemd-udev-trigger.service" ];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = "${pkgs.kmod}/bin/modprobe lkrg";
    };
  };
  boot.extraModprobeConfig = ''
    options lkrg kint_enforce=0 pint_enforce=0 pcfi_enforce=0 smep_enforce=0 smap_enforce=0 umh_validate=0
  '';
}

{ config, lib, pkgs, excalibur-wmi, ... }:
let
  # Kernel paketlerinin callPackage'i: `kernel` ve `kernelModuleMakeFlags`
  # (modul imzalama kurali dahil) otomatik olarak dogru cekirdege baglanir.
  excalibur = config.boot.kernelPackages.callPackage
    ({ stdenv, kernel, kernelModuleMakeFlags }:
      let
        kdir = "${kernel.dev}/lib/modules/${kernel.modDirVersion}/build";
      in
      stdenv.mkDerivation {
        pname = "excalibur-wmi";
        version = "unstable-${excalibur-wmi.shortRev or "git"}";
        src = excalibur-wmi;

        nativeBuildInputs = kernel.moduleBuildDependencies;
        hardeningDisable = [ "pic" "format" ];

        makeFlags = kernelModuleMakeFlags ++ [ "KDIR=${kdir}" ];

        # Repo'nun kendi `install` hedefi `depmod -a` cagiriyor (sandbox'ta bozulur),
        # bu yuzden dogrudan kbuild'in modules_install'i: modulu imzalar ve
        # nixpkgs'in bekledigi excalibur.ko.cer dosyasini uretir.
        installFlags = [
          "-C" kdir
          "M=$(PWD)"
          "INSTALL_MOD_PATH=${placeholder "out"}"
          "INSTALL_MOD_DIR=extra"
          "DEPMOD=true"
        ];
        installTargets = [ "modules_install" ];
      }) { };
in
{
  # AMAC: Casper Excalibur G870 icin WMI surucusu (fan RPM okuma).
  # Cekirdekte casper_wmi yok, bu yuzden out-of-tree derleniyor.
  boot.extraModulePackages = [ excalibur ];
  boot.kernelModules = [ "excalibur" ];
}

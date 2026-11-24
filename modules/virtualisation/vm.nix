# AMAC: Tam izolasyonlu sanal makine (KVM/QEMU + libvirt + virt-manager).
# "Hardcore" isler icin: supheli dosyalar, zararli yazilim, riskli denemeler.
# VM kendi cekirdegiyle calisir; THORNS'un cekirdegini, dosyalarini ve
# donanimini gormez. Gunluk / agir isler icin ileride podman kutusu gelecek.
#
# Neden VirtualBox degil: KVM cekirdegin kendi parcasi (ayri modul derlenmez,
# nixos-unstable'in sik cekirdek guncellemesinde ve LKRG ile sorun cikmaz);
# VirtualBox'in vboxdrv modulu NixOS'ta cekirdek 6.17'de bulunamamisti
# (Ekim 2025). 3D: virgl / Venus.
#
# Ag: libvirt'in "default" NAT agi (virbr0) ACILISTA BASLAMAZ. VM'in
# trafigi THORNS uzerinden yonlendirilir; SKIN 0.1.0'un "routed" zinciri
# kill switch ve LAN katmanini VM'lere de uygular. Agi elle ac:
#   sudo virsh net-start default      (kalici: net-autostart default)

{ config, lib, pkgs, ... }:

{
  virtualisation.libvirtd = {
    enable = true;
    qemu = {
      # Sadece bu makinenin mimarisi (x86_64): yabanci mimari emulatorleri
      # (arm, riscv...) gelmez, diskte ~1 GB yer kalir.
      package = pkgs.qemu_kvm;
      # QEMU root degil, yetkisiz "qemu-libvirtd" kullanicisi olarak calisir:
      # VM'den kacan bir saldirgan root olmaz. Bedeli: ISO'yu ev klasorunden
      # secince virt-manager "izin ver" diye sorar (ACL ekler) -> Evet.
      runAsRoot = false;
      # Sanal TPM 2.0: Windows 11 ve Secure Boot isteyen sistemler icin.
      swtpm.enable = true;
    };
    # THORNS kapanirken calisan VM'ler kapatilir (askiya alinmaz):
    # SSD baska bilgisayarda acilinca yarim kalmis VM durumu kalmasin.
    onShutdown = "shutdown";
    onBoot = "ignore";
  };

  # Pencereli yonetici (VirtualBox benzeri). qemu:///system'e kendisi baglanir.
  programs.virt-manager.enable = true;

  # USB yonlendirme (spiceUSBRedirection) BILINCLI OLARAK KAPALI: VM gercek
  # USB aygitlarina dokunmasin. Gerekirse tek aygit, VM ayarlarindan.

  # VM'in DHCP/DNS istekleri THORNS'a (virbr0) gelir. libvirt kendi kurallarini
  # ekliyor ama NixOS guvenlik duvari yeniden yuklenince silinebiliyor; sadece
  # virbr0 uzerinde, sadece bu iki servis acik.
  networking.firewall.interfaces."virbr0" = {
    allowedUDPPorts = [ 53 67 ];
    allowedTCPPorts = [ 53 ];
  };

  # Sifresiz VM yonetimi (virt-manager her seferinde sifre sormasin).
  users.users."m-arts".extraGroups = [ "libvirtd" ];
}

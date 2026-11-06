{ config, lib, pkgs, ... }:
{
  # Bu modül SADECE bu spesifik laptop (RTX 4050 Max-Q + Raptor Lake-P) için.
  # Bus ID'ler bu makinenin PCI yerleşimine sabittir, başka makinede geçersizdir.

  services.xserver.videoDrivers = [ "modesetting" "nvidia" ];

  # Vulkan (vkd3d/DXVK) needs /dev/nvidia-uvm. Flatpak games can't trigger nvidia-modprobe, so load it at boot.
  boot.kernelModules = [ "nvidia_uvm" ];

  hardware.nvidia = {
    open = true;  # Ada Lovelace (RTX 4050) Turing+ nesli, açık modül önerilir
    modesetting.enable = true;
    nvidiaSettings = true;
    package = config.boot.kernelPackages.nvidiaPackages.stable;

    powerManagement.enable = true;

    prime = {
      offload = {
        enable = true;
        enableOffloadCmd = true;
      };
      intelBusId = "PCI:0@0:2:0";
      nvidiaBusId = "PCI:1@0:0:0";
    };
  };
}

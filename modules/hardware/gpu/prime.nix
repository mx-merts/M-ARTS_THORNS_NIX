# AMAC: PRIME girdisi. SADECE bu laptop (Casper Excalibur: RTX 4050 Max-Q +
# Intel Raptor Lake-P). NVIDIA girdisinin (nvidia.nix) uzerine bu makinenin
# PCI adresleriyle tam PRIME offload ayari eklenir (X11 oturumu dahil).
# Bus ID'ler bu makinenin PCI yerlesimine sabittir, baska makinede gecersizdir;
# baska bir NVIDIA'li laptopta NVIDIA girdisini kullan.
# nvidia-offload komutu nvidia-common.nix'ten gelir.
# nvidia_uvm (Flatpak oyunlarinin Vulkan'i icin) acik modulde NixOS'un
# NVIDIA modulu tarafindan acilista zaten yukleniyor.

{ config, lib, pkgs, ... }:

{
  imports = [ ./nvidia.nix ];

  hardware.nvidia.prime = {
    offload.enable = true;
    intelBusId = "PCI:0@0:2:0";
    nvidiaBusId = "PCI:1@0:0:0";
  };
}

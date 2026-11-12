# AMAC: NVIDIA girdisi. RTX 20 / GTX 16 (Turing, 2018) ve sonrasi, RTX 50
# (Blackwell) ve gelecek nesiller dahil; masaustu ve laptop.
# Acik cekirdek modulu: NVIDIA bu nesiller icin onu oneriyor, RTX 50 ve
# sonrasi ise SADECE onunla calisiyor. Laptoplarda kart bostayken uyur
# (RTD3, Ampere ve sonrasinda surucu varsayilani).
# Daha eski kartlar (GTX 900 / 1000): NVIDIA-L (nvidia-legacy.nix).

{ config, lib, pkgs, ... }:

{
  imports = [ ./nvidia-common.nix ];

  hardware.nvidia = {
    open = true;
    package = config.boot.kernelPackages.nvidiaPackages.stable;
  };
}

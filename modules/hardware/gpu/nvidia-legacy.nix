# AMAC: NVIDIA-L girdisi. GTX 900 (Maxwell, 2014), GTX 1000 (Pascal, 2016)
# ve Volta (2017); masaustu ve laptop.
# NVIDIA bu nesilleri 590 surumunde birakti; 580 dali (LTSB) onlari Agustos
# 2028'e kadar destekliyor. Bu nesiller acik modulu desteklemedigi icin
# kapali (proprietary) modul kullanilir.
# RTX 20 ve sonrasi icin NVIDIA girdisi (nvidia.nix) daha uygun; RTX 50 bu
# girdide CALISMAZ (sadece acik modulle calisir).

{ config, lib, pkgs, ... }:

{
  imports = [ ./nvidia-common.nix ];

  hardware.nvidia = {
    open = false;
    package = config.boot.kernelPackages.nvidiaPackages.legacy_580;
  };
}

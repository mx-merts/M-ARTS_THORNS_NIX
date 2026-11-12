# AMAC: NVIDIA ekran kartli HER makinenin (masaustu ya da laptop) ortak
# ayarlari. NVIDIA (yeni kartlar) ve NVIDIA-L (eski kartlar) girdileri bunu
# import eder; aralarindaki fark sadece surucu dali ve cekirdek modulu turu.

{ config, lib, pkgs, ... }:

{
  # "modesetting": laptoplarda ekrani suren iGPU (Intel/AMD) icin. Tek NVIDIA
  # kartli masaustunde kullanilmaz. Wayland (KWin, Hyprland) GPU duzenini
  # kendisi bulur; bu liste yalnizca X11 oturumunu etkiler.
  services.xserver.videoDrivers = [ "modesetting" "nvidia" ];

  hardware.nvidia = {
    modesetting.enable = true;   # Wayland icin sart (nvidia-drm.modeset=1)
    nvidiaSettings = true;
    # Uyku/uyanmada video bellegini korur (masaustu ve laptop).
    powerManagement.enable = true;
  };

  # Hibrit laptoplarda bir programi NVIDIA kartinda calistirmak icin:
  #   nvidia-offload <program>      (Steam oyun secenegi: nvidia-offload %command%)
  # Makineye ozel busId gerektirmez. Tek NVIDIA kartli masaustunde gerekmez,
  # her sey zaten NVIDIA'da calisir. (NixOS'un kendi komutuyla ayni icerik.)
  environment.systemPackages = [
    (pkgs.writeShellScriptBin "nvidia-offload" ''
      export __NV_PRIME_RENDER_OFFLOAD=1
      export __NV_PRIME_RENDER_OFFLOAD_PROVIDER=NVIDIA-G0
      export __GLX_VENDOR_LIBRARY_NAME=nvidia
      export __VK_LAYER_NV_optimus=NVIDIA_only
      exec "$@"
    '')
  ];
}

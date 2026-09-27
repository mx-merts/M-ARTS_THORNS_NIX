# AMAC: Tek NVIDIA GPU'lu (PRIME/hibrit olmayan) masaustu sistemler icin
#       specialisation icerigi. Ikinci bir GPU olmadigi icin busId pinleme,
#       offload mantigi gerekmiyor - tum driver isini tek NVIDIA karti ustleniyor.
#
#       Kapali (proprietary) kernel modulu bilincli olarak secildi: Maxwell'den
#       Ada Lovelace'e kadar TUM nesillerde calisir (acik modul sadece Turing+
#       destekliyor, eski kartlarda hic calismiyor). Turing ve sonrasinda
#       performans acik modulle birebir ayni (Phoronix R555/R560 testleri),
#       yani agnostiklik icin performanstan odun verilmiyor.

{ config, lib, pkgs, ... }:

{
  services.xserver.videoDrivers = [ "nvidia" ];

  hardware.nvidia = {
    modesetting.enable = true;
    open = false;
    nvidiaSettings = true;
    package = config.boot.kernelPackages.nvidiaPackages.stable;

    # Masaustu = surekli guc kaynagi, laptop tarzi suspend/RTD3 kaygisi yok
    powerManagement.enable = false;
    powerManagement.finegrained = false;
  };

  hardware.graphics = {
    enable = true;
    enable32Bit = true; # 32-bit oyun/uygulama uyumlulugu (Steam vb.)
  };
}

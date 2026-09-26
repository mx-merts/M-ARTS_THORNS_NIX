{ config, lib, pkgs, ... }:
{
  # AMAC: Temel kernel ve giris sertlestirmesi. Hicbir uygulamayi kisitlamaz,
  # sadece bilinen saldiri yuzeylerini kapatir.

  boot.kernel.sysctl = {
    "kernel.kptr_restrict" = 1;
    "kernel.dmesg_restrict" = 1;
    "kernel.yama.ptrace_scope" = 1;
    "net.ipv4.conf.all.rp_filter" = 1;
    "net.ipv4.icmp_echo_ignore_broadcasts" = 1;
    "net.ipv4.tcp_syncookies" = 1;
  };

  security.sudo.execWheelOnly = true;

  # Art arda yanlis sifre denemesinde gecici kilit (hesabi degil, sadece deneme hizini yavaslatir)
  security.pam.services.login.failDelay.delay = 3000000;
}

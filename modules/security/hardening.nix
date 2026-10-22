{ config, lib, pkgs, ... }:
{
  # AMAC: Temel kernel ve giris sertlestirmesi. Hicbir uygulamayi kisitlamaz,
  # sadece bilinen saldiri yuzeylerini kapatir. USB/WiFi/Bluetooth'a dokunmaz.

  boot.kernel.sysctl = {
    # --- Kernel bilgi sizintisi / hata ayiklama ---
    "kernel.kptr_restrict" = 2;
    "kernel.dmesg_restrict" = 1;
    "kernel.yama.ptrace_scope" = 1;

    # --- Kernel yuzeyi ---
    "kernel.unprivileged_bpf_disabled" = 1;
    "net.core.bpf_jit_harden" = 2;
    "kernel.kexec_load_disabled" = 1;
    "dev.tty.ldisc_autoload" = 0;

    # --- Dosya sistemi ---
    "fs.suid_dumpable" = 0;
    "fs.protected_fifos" = 2;
    "fs.protected_regular" = 2;

    # --- Ag ---
    "net.ipv4.conf.all.rp_filter" = 1;
    "net.ipv4.icmp_echo_ignore_broadcasts" = 1;
    "net.ipv4.tcp_syncookies" = 1;
    "net.ipv4.conf.all.accept_redirects" = 0;
    "net.ipv4.conf.default.accept_redirects" = 0;
    "net.ipv6.conf.all.accept_redirects" = 0;
    "net.ipv6.conf.default.accept_redirects" = 0;
    "net.ipv4.conf.all.send_redirects" = 0;
    "net.ipv4.conf.all.accept_source_route" = 0;
    "net.ipv6.conf.all.accept_source_route" = 0;
  };

  # Bellek sertlestirme (kucuk performans maliyeti, reboot sonrasi aktif)
  boot.kernelParams = [
    "init_on_alloc=1"
    "slab_nomerge"
    "randomize_kstack_offset=on"
  ];

  # Kullanilmayan eski protokoller/dosya sistemleri. NixOS'ta /bin/false yok,
  # bu yuzden store'daki coreutils kullaniliyor.
  boot.extraModprobeConfig = lib.concatMapStrings
    (m: "install ${m} ${pkgs.coreutils}/bin/false\n")
    [ "dccp" "sctp" "rds" "tipc" "ax25" "netrom" "rose"
      "cramfs" "freevxfs" "jffs2" ];

  security.sudo.execWheelOnly = true;

  # Art arda yanlis sifre denemesinde gecici kilit (hesabi degil, sadece deneme hizini yavaslatir)
  security.pam.services.login.failDelay.delay = 3000000;
}

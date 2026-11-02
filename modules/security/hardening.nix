{ config, lib, pkgs, ... }:
{
  # AMAC: Kernel sertlestirmesi. Hicbir uygulamayi kisitlamaz;
  # oyun/Steam/Wine/Flatpak bozulmaz. Sadece bilinen saldiri yuzeylerini kapatir.
  # NOT: Ag ile ilgili sysctl'ler (rp_filter, redirect vb.) VPN/DNS duzeni
  # bozulmasin diye bilincli olarak simdilik degistirilmedi -- ag asamasinda ele alinacak.

  boot.kernel.sysctl = {
    # --- Kernel bilgi sizintisi ---
    "kernel.kptr_restrict" = 2;       # kernel bellek adreslerini herkesten gizle (eskiden 1)
    "kernel.dmesg_restrict" = 1;      # kernel loglarini sadece root okuyabilsin
    "kernel.yama.ptrace_scope" = 1;   # bir program baska programin bellegini okuyamasin

    # --- eBPF: yaygin yetki yukseltme yolu ---
    # (isminde "net" gecse de baglantiyla ilgisi yok, sadece BPF JIT'i sertlestirir)
    "kernel.unprivileged_bpf_disabled" = 1;
    "net.core.bpf_jit_harden" = 2;

    # --- Mevcut ag ayarlari (DEGISTIRILMEDI) ---
    "net.ipv4.conf.all.rp_filter" = 1;
    "net.ipv4.icmp_echo_ignore_broadcasts" = 1;
    "net.ipv4.tcp_syncookies" = 1;
  };

  # Kernel imajini ve /dev/mem'i yazmaya kapatir, kexec'i engeller.
  # Hibernate'i kapatir ama zaten zram kullandigimiz icin hibernate yok.
  security.protectKernelImage = true;

  security.sudo.execWheelOnly = true;

  # TTY girisinde yanlis sifre sonrasi 3 sn bekleme (hesap kilidi DEGIL)
  security.pam.services.login.failDelay.delay = 3000000;
}

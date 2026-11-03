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
    "kernel.perf_event_paranoid" = 3; # perf olaylari sadece root (kernel bilgi sizintisi)

    # --- eBPF: yaygin yetki yukseltme yolu ---
    # (isminde "net" gecse de baglantiyla ilgisi yok, sadece BPF JIT'i sertlestirir)
    "kernel.unprivileged_bpf_disabled" = 1;
    "net.core.bpf_jit_harden" = 2;

    # --- Yerel yetki yukseltme yollari ---
    "dev.tty.ldisc_autoload" = 0;           # TTY line discipline otomatik yuklenmesin (CVE kaynagi)
    "vm.unprivileged_userfaultfd" = 0;      # userfaultfd: kernel exploit'lerinin sevdigi arac
    "fs.protected_fifos" = 2;               # /tmp gibi paylasilan dizinlerde FIFO/dosya
    "fs.protected_regular" = 2;             #   tuzaklarina karsi koruma
    "fs.suid_dumpable" = 0;                 # setuid programlar core dump birakmasin

    # --- Mevcut ag ayarlari (DEGISTIRILMEDI) ---
    "net.ipv4.conf.all.rp_filter" = 1;
    "net.ipv4.icmp_echo_ignore_broadcasts" = 1;
    "net.ipv4.tcp_syncookies" = 1;
  };

  # --- Acilis parametreleri (Linux kernel'in kendi ozellikleri) ---
  boot.kernelParams = [
    "slab_nomerge"               # bellek havuzlarini ayri tut: heap exploit'lerini zorlastirir
    "init_on_alloc=1"            # ayrilan bellegi sifirla: eski veri sizmasin
    "page_alloc.shuffle=1"       # sayfa ayirma sirasini rastgele yap
    "randomize_kstack_offset=on" # her sistem cagrisinda kernel yigini adresi rastgele
    "vsyscall=none"              # eski vsyscall sayfasi kapali (2011 oncesi glibc'li
                                 # cok eski 64-bit binary'ler disinda etkisi yok)
    "debugfs=off"                # kernel debug dosya sistemi kapali (bilgi sizintisi)
  ];
  # init_on_free=1 bilincli olarak EKLENMEDI: oyunlarda performans maliyeti var.

  # --- Kullanilmayan eski protokol ve dosya sistemi modulleri ---
  # Otomatik yuklenmezler (gecmiste cok sayida kernel acigi cikti). Gercekten
  # gerekirse "sudo modprobe <ad>" ile elle yuklenebilirler.
  # VPN/WireGuard, TCP/UDP, USB bellekler (vfat/exfat/ntfs3), udf, squashfs
  # (AppImage) ETKILENMEZ.
  boot.blacklistedKernelModules = [
    # Ag protokolleri (kimse kullanmiyor, CVE gecmisi kotu)
    "dccp" "sctp" "rds" "tipc" "n-hdlc" "ax25" "netrom" "x25" "rose"
    "decnet" "econet" "af_802154" "ipx" "appletalk" "psnap" "p8023" "p8022"
    "can" "atm"
    # Eski/egzotik dosya sistemleri
    "cramfs" "freevxfs" "jffs2" "hfs" "hfsplus"
    # FireWire: eski port, DMA saldirisi yolu
    "firewire-core" "firewire-ohci" "firewire-sbp2"
  ];

  # Kernel imajini ve /dev/mem'i yazmaya kapatir, kexec'i engeller.
  # Hibernate'i kapatir ama zaten zram kullandigimiz icin hibernate yok.
  security.protectKernelImage = true;

  # sudo-rs: sadece wheel grubu calistirabilsin (sudo -> sudo-rs gecisi, bkz. hosts/portable)
  security.sudo-rs.execWheelOnly = true;

  # TTY girisinde yanlis sifre sonrasi 3 sn bekleme (hesap kilidi DEGIL)
  security.pam.services.login.failDelay.delay = 3000000;

  # fwupd KAPALI: Plasma varsayilan olarak aciyor. Sistem okul/baskasinin
  # bilgisayarina takildiginda KDE Discover O MAKINENIN firmware'ini
  # guncellemeyi onerebilir -- baskasinin donanimina firmware basilmasin.
  # Kendi laptopunun firmware guncellemeleri Windows tarafindan yapilir.
  services.fwupd.enable = lib.mkForce false;
}

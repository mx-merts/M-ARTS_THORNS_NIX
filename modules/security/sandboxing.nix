{ config, lib, pkgs, ... }:
{
  # AMAC: Uygulama izolasyonu.
  # Firejail KALDIRILDI: setuid-root calisan buyuk bir binary, gecmisinde
  # yetki yukseltme CVE'leri var (or. CVE-2022-31214) ve hicbir uygulamaya
  # uygulanmiyordu -- koruma vermeden saldiri yuzeyi ekliyordu.
  # Izolasyon: Flatpak (bubblewrap) + ileride LRUN/JRUN icin bubblewrap kalkani.
  programs.firejail.enable = false;
}

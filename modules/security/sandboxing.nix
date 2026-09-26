{ config, lib, pkgs, ... }:
{
  # AMAC: Firejail hazir bekler, HICBIR UYGULAMAYA OTOMATIK UYGULANMAZ.
  # Kullanim: istedigin an "firejail firefox" gibi elle baslatirsin.
  programs.firejail.enable = true;
}

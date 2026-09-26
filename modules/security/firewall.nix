{ config, lib, pkgs, ... }:
{
  # AMAÇ: Güvenlik katmanı. Yeni firewall kuralı/port açma buraya eklenir.
  networking.firewall.enable = true;
}

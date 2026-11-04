{ config, lib, pkgs, ... }:
{
  # AMAÇ: Güvenlik katmanı. Yeni firewall kuralı/port açma buraya eklenir.
  networking.firewall.enable = true;

  # Ping'e cevap verme: basit "ping sweep" taramalarinda gorunme.
  # (Ayni agdakiler ARP ile yine gorebilir; ucuz ama kucuk bir kazanc.)
  networking.firewall.allowPing = false;
}

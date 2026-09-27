# AMAC: Sistem geneli şifreli DNS (DoH, Cloudflare birincil / Quad9 yedek) + yerel
# cache. Tek DNS otoritesi burasıdır. VPN arayüzü kendi DNS'ini push etmeyecek
# şekilde ayarlanacağı için (bkz. vpn.nix), services.resolved.settings.Resolve.Domains
# = "~." global catch-all olarak tek otoriteyi garanti eder.
# NOT: networking.networkmanager.dns burada AYARLANMAZ — services.resolved.enable
# zaten bunu "systemd-resolved" olarak zorluyor, bu da doğru davranış (NetworkManager
# resolv.conf'u kendi yazmıyor, DNS bilgisini resolved'a devrediyor).

{ config, lib, pkgs, ... }:

{
  services.dnscrypt-proxy = {
    enable = true;
    settings = {
      listen_addresses = [ "127.0.0.1:53" ];
      # Cloudflare birincil (DoH), Quad9 yedek — tek nokta bağımlılığı yok
      server_names = [ "cloudflare" "quad9-dnscrypt-ip4-filter-pri" ];

      # Yerel cache — tekrar sorgular RAM'den cevaplanır, asıl hız kazancı burada
      cache = true;
      cache_size = 4096;
      cache_min_ttl = 2400;
      cache_max_ttl = 86400;

      lb_strategy = "p2";
    };
  };

  # systemd-resolved: tek otorite, tüm sorgular 127.0.0.1'e (dnscrypt-proxy)
  services.resolved = {
    enable = true;
    settings.Resolve = {
      DNSSEC = "false"; # dnscrypt-proxy zaten kendi doğrulamasını yapıyor
      Domains = [ "~." ];
    };
  };

  networking.nameservers = [ "127.0.0.1" ];
}

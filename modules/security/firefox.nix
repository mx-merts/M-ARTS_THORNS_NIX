{ config, lib, pkgs, ... }:
{
  # AMAC: Firefox guvenlik politikalari (kurumsal "policies.json").
  # Gunluk kullanimi bozmayan, deklaratif, her makinede ayni gelen ayarlar.
  # NOT: Bu politikalar NOCER uzerinden uygulanir (nocer/default.nix):
  # NOCER bunlara sadece aktif sertifikalari ekler. Kontrol: about:policies
  # Ag ayarlarina (DNS-over-HTTPS, proxy) bilincli olarak DOKUNULMADI.

  programs.firefox.policies = {
    DisableTelemetry = true;
    DisableFirefoxStudies = true;
    DisablePocket = true;

    # HTTP siteler once uyari sayfasi gosterir; "HTTP siteye devam et" ile
    # tek tikla gecilir (okul wifi giris sayfalari dahil). Kilitli degil.
    HttpsOnlyMode = "enabled";

    EnableTrackingProtection = {
      Value = true;
      Cryptomining = true;
      Fingerprinting = true;
    };

    # uBlock Origin: reklam + zararli/phishing alan adi listeleri.
    # Zorunlu kurulu (bir zararli eklentiyi kaldirip yerine gecemez).
    ExtensionSettings."uBlock0@raymondhill.net" = {
      installation_mode = "force_installed";
      install_url = "https://addons.mozilla.org/firefox/downloads/latest/ublock-origin/latest.xpi";
    };

    # Kilitli: bir zararli prefs.js'yi degistirerek bunlari KAPATAMAZ.
    Preferences = {
      # Google Safe Browsing: zararli site / phishing / zararli indirme uyarisi
      "browser.safebrowsing.malware.enabled" = { Value = true; Status = "locked"; };
      "browser.safebrowsing.phishing.enabled" = { Value = true; Status = "locked"; };
      "browser.safebrowsing.downloads.enabled" = { Value = true; Status = "locked"; };
      # Sahte alan adlari (or. kiril "а" ile yazilmis "pаypal.com") punycode gorunur
      "network.IDN_show_punycode" = { Value = true; Status = "locked"; };
      # PDF icindeki JavaScript kapali (PDF exploit'leri); PDF'ler yine acilir
      "pdfjs.enableScripting" = { Value = false; Status = "locked"; };
    };
  };
}

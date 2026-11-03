{ config, lib, pkgs, ... }:

let
  cfg = config.programs.nocer;
  inherit (lib) mkIf mkEnableOption mkOption types;

  nocer = pkgs.callPackage ./package.nix {};

  # Nix'in Firefox politikalari (programs.firefox.policies: telemetri, uBlock,
  # HTTPS-Only vb.). NOCER bunlari bozmadan sadece "Certificates" ekler.
  firefoxBasePolicy = (pkgs.formats.json { }).generate "firefox-base-policies.json" {
    inherit (config.programs.firefox) policies;
  };

  rebuildBundle = pkgs.writeShellApplication {
    name = "nocer-rebuild-bundle";
    runtimeInputs = [ pkgs.coreutils ];
    text = ''
      set -euo pipefail

      OUT="${cfg.bundlePath}"
      STATE="${cfg.stateDir}"

      tmp="$(mktemp)"
      trap 'rm -f "$tmp"' EXIT

      cat /etc/ssl/certs/ca-certificates.crt > "$tmp"

      if [ -d "$STATE/enabled" ]; then
        shopt -s nullglob
        for f in "$STATE"/enabled/*; do
          [ -f "$f" ] || continue
          printf '\n' >> "$tmp"
          cat "$f" >> "$tmp"
        done
      fi

      install -m 0644 "$tmp" "$OUT"
    '';
  };
in
{
  options.programs.nocer = {
    enable = mkEnableOption "NOCER — NixOS runtime certificate manager";

    stateDir = mkOption {
      type = types.str;
      default = "/var/lib/nocer";
      description = "Sertifikaların ve metadata'nın tutulduğu dizin.";
    };

    bundlePath = mkOption {
      type = types.str;
      default = "/run/nocer-ca-bundle.crt";
      description = "Runtime'da üretilen birleşik CA bundle.";
    };

    setGlobalEnv = mkOption {
      type = types.bool;
      default = true;
      description = "SSL_CERT_FILE ve benzeri env değişkenlerini global ayarla.";
    };

    manageFirefoxPolicy = mkOption {
      type = types.bool;
      default = true;
      description = ''
        /etc/firefox/policies/policies.json'ı NOCER'in urettigi dosyaya bagla.
        Dosya = programs.firefox.policies (Nix) + aktif sertifikalar (NOCER),
        yani Nix politikalari kaybolmaz.
      '';
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ nocer pkgs.fzf pkgs.openssl ];

    # GUVENLIK: Tum state SADECE root'a yazilabilir.
    # Onceden 0775 root:wheel idi -- kullanici yetkisiyle calisan HERHANGI bir
    # program sudo'suz sahte CA ekleyebiliyor veya Firefox politikalarini
    # (zorunlu eklenti, proxy vb.) degistirebiliyordu. Artik degisiklik icin
    # "sudo nocer" gerekir; okuma (nocer list) herkese acik.
    # "Z" satiri: daha once kullanici tarafindan olusturulmus dosyalarin
    # sahipligini de geriye donuk olarak root'a cevirir.
    systemd.tmpfiles.rules = [
      "d ${cfg.stateDir}              0755 root root -"
      "d ${cfg.stateDir}/certs        0755 root root -"
      "d ${cfg.stateDir}/enabled      0755 root root -"
      "f ${cfg.stateDir}/index.json   0644 root root -"
      "f ${cfg.stateDir}/firefox-policies.json 0644 root root -"
      "Z ${cfg.stateDir}              -    root root -"
    ];

    # Firefox politikalari: eskiden tmpfiles "L+" ile /etc dosyasinin USTUNE
    # yaziliyordu ve programs.firefox'un kendi politikalari (DisableAppUpdate
    # vb.) sessizce kayboluyordu. Artik:
    #   /etc/nocer/firefox-base-policies.json  <- Nix politikalari (salt-okunur)
    #   /etc/firefox/policies/policies.json    -> NOCER'in birlestirdigi dosya
    environment.etc = lib.mkIf cfg.manageFirefoxPolicy {
      "nocer/firefox-base-policies.json".source = firefoxBasePolicy;
      "firefox/policies/policies.json".source = lib.mkForce "${cfg.stateDir}/firefox-policies.json";
    };

    # Birlesik dosyayi acilista ve Nix politikalari her degistiginde yeniden
    # uretir (nocer her calismada politikayi yazar; "list" hicbir seyi degistirmez).
    systemd.services.nocer-firefox-policy = lib.mkIf cfg.manageFirefoxPolicy {
      description = "Merge Nix Firefox policies with NOCER certificates";
      wantedBy = [ "multi-user.target" ];
      after = [ "systemd-tmpfiles-setup.service" ];
      restartTriggers = [ firefoxBasePolicy ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStart = "${nocer}/bin/nocer list";
      };
    };

    systemd.services.nocer-rebuild = {
      description = "Rebuild NOCER CA bundle";
      wantedBy = [ "multi-user.target" ];
      before = [ "multi-user.target" ];
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${rebuildBundle}/bin/nocer-rebuild-bundle";
      };
    };

    systemd.paths.nocer-rebuild = {
      wantedBy = [ "multi-user.target" ];
      pathConfig = {
        PathChanged = "${cfg.stateDir}/enabled";
        PathModified = "${cfg.stateDir}/enabled";
        Unit = "nocer-rebuild.service";
      };
    };

    # PAM üzerinden TÜM shell'lere dağıt (login olsun olmasın)
    environment.sessionVariables = lib.mkMerge [
      { NOCER_STATE_DIR   = cfg.stateDir;
        NOCER_BUNDLE_PATH = cfg.bundlePath; }
      (lib.mkIf cfg.setGlobalEnv {
        SSL_CERT_FILE       = cfg.bundlePath;
        NIX_SSL_CERT_FILE   = cfg.bundlePath;
        CURL_CA_BUNDLE      = cfg.bundlePath;
        GIT_SSL_CAINFO      = cfg.bundlePath;
        REQUESTS_CA_BUNDLE  = cfg.bundlePath;
        NODE_EXTRA_CA_CERTS = cfg.bundlePath;
        DENO_CERT           = cfg.bundlePath;
        AWS_CA_BUNDLE       = cfg.bundlePath;
        PIP_CERT            = cfg.bundlePath;
      })
    ];
  };
}

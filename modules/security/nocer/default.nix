{ config, lib, pkgs, ... }:

let
  cfg = config.programs.nocer;
  inherit (lib) mkIf mkEnableOption mkOption types;

  nocer = pkgs.callPackage ./package.nix {};

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
        /etc/firefox/policies/policies.json'ı writable bir konuma symlink'le.
        UYARI: programs.firefox.policies kullanıyorsan bunu false yap.
      '';
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ nocer pkgs.fzf pkgs.openssl ];

    systemd.tmpfiles.rules = [
      "d ${cfg.stateDir}              0775 root wheel -"
      "d ${cfg.stateDir}/certs        0775 root wheel -"
      "d ${cfg.stateDir}/enabled      0775 root wheel -"
      "f ${cfg.stateDir}/index.json   0664 root wheel -"
      "f ${cfg.stateDir}/firefox-policies.json 0664 root wheel -"
    ] ++ lib.optionals cfg.manageFirefoxPolicy [
      "d /etc/firefox                0755 root root -"
      "d /etc/firefox/policies       0755 root root -"
      "L+ /etc/firefox/policies/policies.json - - - - ${cfg.stateDir}/firefox-policies.json"
    ];

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

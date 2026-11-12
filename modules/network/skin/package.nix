{ lib
, rustPlatform
, makeWrapper
, coreutils
, iproute2
, nftables
, networkmanager
, systemd
, wireguard-tools
, libnotify
, warp
}:

rustPlatform.buildRustPackage {
  pname = "skin";
  version = "0.0.2";

  src = ./source;
  cargoLock.lockFile = ./source/Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  # Calisma aninda cagrilan komutlar:
  #   date                  saatler (coreutils)
  #   ip                    ag gecidi, kalici MAC (iproute2)
  #   nft                   KILLSWITCH + LAN tablosu "inet skin" (nftables)
  #   nmcli                 baglanti, SSID, gecici MAC degisimi (networkmanager)
  #   systemctl, resolvectl dnscrypt-proxy ac/kapa, ag DNS'i (systemd)
  #   wg                    Proton'un WireGuard uc noktalari (wireguard-tools)
  #   warp-cli              TUNNEL (sistemdeki WARP paketi, warp.nix)
  #   notify-send           bildirimler (libnotify)
  # Komut "SKIN"; kucuk harfle "skin" de calissin diye bir kisayol eklenir.
  postInstall = ''
    wrapProgram $out/bin/SKIN \
      --prefix PATH : ${lib.makeBinPath [ coreutils iproute2 nftables networkmanager systemd wireguard-tools libnotify warp ]}
    ln -s SKIN $out/bin/skin
  '';

  meta = with lib; {
    description = "SKIN-0.0.2: network membrane for THORNS (SEAL, SEVER, ROT per layer)";
    homepage = "https://github.com/mx-merts/M-ARTS_THORNS_NIX";
    license = licenses.cc-by-nc-sa-40;
    mainProgram = "SKIN";
  };
}

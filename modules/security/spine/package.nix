{ lib
, rustPlatform
, makeWrapper
, systemd
, libnotify
}:

rustPlatform.buildRustPackage {
  pname = "spine";
  version = "0.2.0";

  src = ./source;
  cargoLock.lockFile = ./source/Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  # The unit tests (cargo test, 19 of them) were run during development. They
  # read /etc/passwd and use temp directories, which the Nix build sandbox does
  # not reliably provide, so they are not part of the package build.
  doCheck = false;

  # journalctl (ClamAV journal) and notify-send (desktop notifications).
  postInstall = ''
    wrapProgram $out/bin/spine \
      --prefix PATH : ${lib.makeBinPath [ systemd libnotify ]}
  '';

  meta = with lib; {
    description = "SPINE: local security event hub (observes and reports, never blocks)";
    homepage = "https://github.com/mx-merts/M-ARTS_THORNS_NIX";
    license = licenses.cc-by-nc-sa-40;
    mainProgram = "spine";
  };
}

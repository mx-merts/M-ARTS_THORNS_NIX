{ lib
, rustPlatform
, pkg-config
, openssl
, fzf
, makeWrapper
}:

rustPlatform.buildRustPackage {
  pname = "nocer";
  version = "0.1.0";

  src = ./source;

  cargoLock.lockFile = ./source/Cargo.lock;

  nativeBuildInputs = [ pkg-config makeWrapper ];
  buildInputs = [ openssl ];

  postInstall = ''
    wrapProgram $out/bin/nocer \
      --prefix PATH : ${lib.makeBinPath [ fzf ]}
  '';

  meta = with lib; {
    description = "NixOS runtime certificate manager";
    homepage = "https://github.com/mx-merts/M-ARTS_THORNS_NIX";
    license = licenses.mit;
    mainProgram = "nocer";
  };
}

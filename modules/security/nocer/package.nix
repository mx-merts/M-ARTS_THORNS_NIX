{ lib
, rustPlatform
, makeWrapper
, openssl
, fzf
, findutils
}:

rustPlatform.buildRustPackage {
  pname = "nocer";
  version = "0.1.0";

  src = ./source;

  cargoLock.lockFile = ./source/Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  # Calisma aninda cagrilan komutlar: fzf + find (dosya secici),
  # openssl (DER -> PEM cevirme). Kutuphane olarak openssl'e baglanmaz.
  postInstall = ''
    wrapProgram $out/bin/nocer \
      --prefix PATH : ${lib.makeBinPath [ fzf findutils openssl ]}
  '';

  meta = with lib; {
    description = "NixOS runtime certificate manager";
    homepage = "https://github.com/mx-merts/M-ARTS_THORNS_NIX";
    license = licenses.mit;
    mainProgram = "nocer";
  };
}

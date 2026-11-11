{ lib
, rustPlatform
, makeWrapper
, openssl
, fzf
, findutils
, nssTools
, util-linux
}:

rustPlatform.buildRustPackage {
  pname = "nocer";
  version = "0.0.2";

  src = ./source;

  cargoLock.lockFile = ./source/Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  # Calisma aninda cagrilan komutlar:
  #   fzf + find   dosya secici
  #   openssl      DER -> PEM cevirme, parmak izi
  #   certutil     Firefox profilleri ve ~/.pki/nssdb (NSS depolari)
  #   setpriv      certutil'i sudo altinda kullanicinin kimligiyle calistirma
  # Kutuphane olarak openssl'e baglanmaz.
  postInstall = ''
    wrapProgram $out/bin/nocer \
      --prefix PATH : ${lib.makeBinPath [ fzf findutils openssl nssTools util-linux ]}
  '';

  meta = with lib; {
    description = "NOCER-0.0.2: NixOS runtime certificate manager";
    homepage = "https://github.com/mx-merts/M-ARTS_THORNS_NIX";
    license = licenses.mit;
    mainProgram = "nocer";
  };
}

{ lib
, rustPlatform
, makeWrapper
, kmod
, coreutils
, bluez
, networkmanager
, udisks
, libnotify
, hwdata
}:

rustPlatform.buildRustPackage {
  pname = "nerv";
  version = "0.0.2";

  src = ./source;
  cargoLock.lockFile = ./source/Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  # Calisma aninda cagrilan komutlar:
  #   modprobe       surucu bulma (-R), kara liste (-c), yukleme (-b)
  #   date           acilis taramasinin saati
  #   bluetoothctl   Bluetooth acik mi / ac / kapat
  #   nmcli          Wi-Fi / Ethernet hangi baglantida
  #   udisksctl      USB diski guvenli cikarma (DISABLE)
  #   notify-send    giris bildirimleri
  # NERV_HWDATA: pci.ids ve usb.ids (parca adlari) buradan okunur.
  # Komut "NERV"; kucuk harfle "nerv" de calissin diye bir kisayol eklenir.
  postInstall = ''
    wrapProgram $out/bin/NERV \
      --prefix PATH : ${lib.makeBinPath [ kmod coreutils bluez networkmanager udisks libnotify ]} \
      --set NERV_HWDATA ${hwdata}/share/hwdata
    ln -s NERV $out/bin/nerv
  '';

  meta = with lib; {
    description = "NERV-0.0.2: hardware nervous system for THORNS (scan, report, bind, control)";
    homepage = "https://github.com/mx-merts/M-ARTS_THORNS_NIX";
    license = licenses.cc-by-nc-sa-40;
    mainProgram = "NERV";
  };
}

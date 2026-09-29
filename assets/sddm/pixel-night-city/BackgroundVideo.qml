import QtQuick
import QtQuick.Window

// Orijinal qylock temasi MediaPlayer ile bg.mp4 oynatiyordu.
// NixOS portable sistem icin statik PNG arka plana cevrildi.
// Dosya adi "BackgroundVideo.qml" olarak kaldi cunku Main.qml bunu
// sabit yoldan cagiriyor; icerigi degistirdik, ismi degil.
Item {
    anchors.fill: parent
    Image {
        anchors.fill: parent
        source: "bg.png"
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        cache: false
    }
}

# AMAC: SKlauncher'i steam-run uzerinden calistirir. steam-run, NixOS'un FHS
# uyumsuzlugunu cozen resmi cozum (Steam icin gelistirilmis). JavaFX'in
# ihtiyac duydugu libXxf86vm, libgthread gibi kutuphaneler hazir gelir.
#
# bwrap chdir hatasini onlemek icin script /tmp'ye gecis yapar.
# Steam zaten modules/apps/gaming.nix'te aktif (programs.steam.enable).

{ config, lib, pkgs, ... }:

let
  sklauncher = pkgs.writeShellScriptBin "sklauncher" ''
    cd /tmp
    exec ${pkgs.steam-run}/bin/steam-run ${pkgs.jdk21}/bin/java \
      -jar /home/m-arts/Desktop/SKlauncher-3.2.18.jar "$@"
  '';
in
{
  environment.systemPackages = [ sklauncher ];
}

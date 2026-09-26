{ config, lib, pkgs, ... }:
{
  # AMAÇ: Ses donanımı yönetimi (pipewire). Donanım-agnostik, her makinede güvenli.
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };
}

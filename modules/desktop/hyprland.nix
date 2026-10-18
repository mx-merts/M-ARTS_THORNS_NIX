{ config, lib, pkgs, ... }:

let
  # Başlangıç config'i. Yalnızca ~/.config/hypr/hyprland.conf YOKSA kopyalanır.
  starterConfig = pkgs.writeText "hyprland-starter.conf" ''
    # ── Başlangıç config'i (modules/desktop/hyprland.nix tarafından bir kez kopyalandı) ──
    # Bu dosyayı serbestçe düzenle; kaydedince Hyprland otomatik yeniler.
    # Sıfırlamak için: dosyayı sil, rebuild yap.

    monitor = , preferred, auto, 1

    $mod      = SUPER
    $terminal = kitty
    $browser  = firefox
    $files    = dolphin
    $menu     = fuzzel

    input {
        kb_layout = tr
        kb_variant =
        follow_mouse = 1
        touchpad {
            natural_scroll = true
        }
    }

    general {
        gaps_in = 4
        gaps_out = 8
        border_size = 2
        layout = dwindle
    }

    misc {
        disable_hyprland_logo = true
    }

    exec-once = waybar
    exec-once = mako
    exec-once = systemctl --user start hyprpolkitagent

    # ── Programlar ──
    bind = $mod, Return, exec, $terminal
    bind = $mod, B, exec, $browser
    bind = $mod, E, exec, $files
    bind = $mod, D, exec, $menu
    bind = $mod, C, exec, $terminal micro ~/.config/hypr/hyprland.conf

    # ── Pencere ──
    bind = $mod, Q, killactive
    bind = $mod, F, fullscreen
    bind = $mod, V, togglefloating
    bind = $mod SHIFT, E, exit

    bind = $mod, left,  movefocus, l
    bind = $mod, right, movefocus, r
    bind = $mod, up,    movefocus, u
    bind = $mod, down,  movefocus, d

    bindm = $mod, mouse:272, movewindow
    bindm = $mod, mouse:273, resizewindow

    # ── Çalışma alanları ──
    bind = $mod, 1, workspace, 1
    bind = $mod, 2, workspace, 2
    bind = $mod, 3, workspace, 3
    bind = $mod, 4, workspace, 4
    bind = $mod, 5, workspace, 5
    bind = $mod SHIFT, 1, movetoworkspace, 1
    bind = $mod SHIFT, 2, movetoworkspace, 2
    bind = $mod SHIFT, 3, movetoworkspace, 3
    bind = $mod SHIFT, 4, movetoworkspace, 4
    bind = $mod SHIFT, 5, movetoworkspace, 5

    # ── Ekran görüntüsü (seçili alan → panoya) ──
    bind = , Print, exec, grim -g "$(slurp)" - | wl-copy

    # ── Ses / parlaklık ──
    bindel = , XF86AudioRaiseVolume, exec, wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+
    bindel = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
    bindl  = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
    bindel = , XF86MonBrightnessUp,   exec, brightnessctl set 5%+
    bindel = , XF86MonBrightnessDown, exec, brightnessctl set 5%-
  '';
in
{
  # AMAÇ: Hyprland (Wayland compositor). Sistem tarafı burada, kullanıcı config'i
  # ~/.config/hypr/hyprland.conf içinde elle yönetilir (Home Manager gelene kadar).
  programs.hyprland = {
    enable = true;
    xwayland.enable = true;
  };

  # Dosya seçici vb. için GTK portalı
  xdg.portal.extraPortals = [ pkgs.xdg-desktop-portal-gtk ];

  environment.systemPackages = with pkgs; [
    waybar               # üst bar
    fuzzel               # uygulama başlatıcı
    mako                 # bildirimler
    hyprpolkitagent      # polkit penceresi
    wl-clipboard         # wl-copy / wl-paste
    grim slurp           # ekran görüntüsü
    brightnessctl        # parlaklık
    cliphist             # pano geçmişi (Serpantinum)
    playerctl            # medya tuşları (Serpantinum)
    kdePackages.dolphin  # dosya yöneticisi
  ];

  # config'i yalnızca yoksa kopyala ("C" = copy if not exists)
  systemd.tmpfiles.rules = [
    "d /home/m-arts/.config        0700 m-arts users -"
    "d /home/m-arts/.config/hypr   0755 m-arts users -"
    "C /home/m-arts/.config/hypr/hyprland.conf 0644 m-arts users - ${starterConfig}"
  ];
}

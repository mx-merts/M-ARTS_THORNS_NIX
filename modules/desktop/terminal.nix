{ config, lib, pkgs, ... }:

{
  # ─── Starship (end-4/dots-hyprland tarzı) ───
  # Ayarlar ayni klasordeki starship.toml'dan okunur (powerline ikonlari
  # bozulmasin diye dosya elle degil, cp ile kopyalandi).
  # Kaynak: https://github.com/end-4/dots-hyprland
  programs.starship = {
    enable = true;
    settings = lib.importTOML ./starship.toml;
  };

  environment.systemPackages = [ pkgs.kitty ];

  # ─── Kitty (Catppuccin Mocha + Şeffaflık + Blur) ───
  environment.etc."xdg/kitty/kitty.conf".text = ''
    font_family      JetBrainsMono Nerd Font
    font_size        12.0
    background_opacity 0.65
    background_blur  64
    window_padding_width 10
    confirm_os_window_close 0
    cursor_shape     beam
    cursor_trail     1
    enable_audio_bell no
    tab_bar_style    powerline
    tab_powerline_style slanted

    # Catppuccin Mocha
    foreground            #cdd6f4
    background            #1e1e2e
    selection_foreground  #1e1e2e
    selection_background  #f5e0dc
    cursor                #f5e0dc
    cursor_text_color     #1e1e2e
    url_color             #f5e0dc
    active_border_color   #b4befe
    inactive_border_color #6c7086
    bell_border_color     #f9e2af
    active_tab_foreground   #1e1e2e
    active_tab_background   #89b4fa
    active_tab_font_style   bold
    inactive_tab_foreground #cdd6f4
    inactive_tab_background #313244
    inactive_tab_font_style normal

    # Kısayollar (end-4/dots-hyprland'dan)
    map ctrl+c           copy_or_interrupt
    map page_up          scroll_page_up
    map page_down        scroll_page_down
    map ctrl+plus        change_font_size all +1
    map ctrl+equal       change_font_size all +1
    map ctrl+kp_add      change_font_size all +1
    map ctrl+minus       change_font_size all -1
    map ctrl+underscore  change_font_size all -1
    map ctrl+kp_subtract change_font_size all -1
    map ctrl+0           change_font_size all 0
    map ctrl+kp_0        change_font_size all 0

    color0  #45475a
    color8  #585b70
    color1  #f38ba8
    color9  #f38ba8
    color2  #a6e3a1
    color10 #a6e3a1
    color3  #f9e2af
    color11 #f9e2af
    color4  #89b4fa
    color12 #89b4fa
    color5  #f5c2e7
    color13 #f5c2e7
    color6  #94e2d5
    color14 #94e2d5
    color7  #bac2de
    color15 #a6adc8
  '';

  # ─── Nerd Fonts ───
  fonts.packages = with pkgs; [
    nerd-fonts.jetbrains-mono
    nerd-fonts.fira-code
  ];
}

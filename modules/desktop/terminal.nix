{ config, lib, pkgs, ... }:

{
  # ─── Starship (Catppuccin Mocha) ───
  programs.starship = {
    enable = true;
    settings = {
      add_newline = true;
      palette = "catppuccin_mocha";
      format = lib.concatStrings [
        "$directory"
        "$git_branch"
        "$git_status"
        "$nix_shell"
        "$cmd_duration"
        "$line_break"
        "$character"
      ];

      directory = {
        style = "bold mauve";
        truncation_length = 3;
        truncation_symbol = "…/";
      };
      git_branch = { style = "bold lavender"; symbol = " "; };
      git_status = { style = "bold red"; };
      nix_shell = {
        symbol = " ";
        format = "[$symbol$state]($style) ";
        style = "bold sky";
      };
      cmd_duration = {
        min_time = 2000;
        format = "[ $duration]($style) ";
        style = "yellow";
      };
      character = {
        success_symbol = "[❯](bold green)";
        error_symbol = "[❯](bold red)";
      };

      palettes.catppuccin_mocha = {
        rosewater = "#f5e0dc";
        flamingo  = "#f2cdcd";
        pink      = "#f5c2e7";
        mauve     = "#cba6f7";
        red       = "#f38ba8";
        maroon    = "#eba0ac";
        peach     = "#fab387";
        yellow    = "#f9e2af";
        green     = "#a6e3a1";
        teal      = "#94e2d5";
        sky       = "#89dceb";
        sapphire  = "#74c7ec";
        blue      = "#89b4fa";
        lavender  = "#b4befe";
        text      = "#cdd6f4";
        subtext1  = "#bac2de";
        subtext0  = "#a6adc8";
        overlay2  = "#9399b2";
        overlay1  = "#7f849c";
        overlay0  = "#6c7086";
        surface2  = "#585b70";
        surface1  = "#45475a";
        surface0  = "#313244";
        base      = "#1e1e2e";
        mantle    = "#181825";
        crust     = "#11111b";
      };
    };
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
    cursor_shape     block
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

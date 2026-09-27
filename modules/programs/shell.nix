# AMAC: Zsh + Oh My Zsh (populer, plugin/tema ekosistemi genis, baskalarindan
# kopyalamak kolay) + fastfetch (her yeni terminalde otomatik sistem bilgisi,
# ozel ASCII logo + sade/duzenli bilgi formati). Logo ve config /etc altina
# deklaratif olarak gomulu -- elle duzenlenen /etc dosyalarina bagimli degil,
# git'te kayitli, her rebuild'de garantili ayni.

{ config, lib, pkgs, ... }:

{
  programs.zsh = {
    enable = true;
    ohMyZsh = {
      enable = true;
      theme = "robbyrussell";
      plugins = [ "git" "sudo" "history-substring-search" ];
    };
    interactiveShellInit = ''
      fastfetch --config /etc/fastfetch/config.jsonc
    '';
  };

  environment.systemPackages = with pkgs; [
    fastfetch
  ];

  users.users.m-arts.shell = pkgs.zsh;

  environment.etc."fastfetch/logo.txt".text = ''
.             .  .                        `\     \
|             #  f                        t`    /;       ;\   .Kf.       ;
L.           .#  f       ;    ;    .K.    ; `\     ., \  /#t(       #
EW:        ,ft  t     ##    t;    `f#\    \  ;   /` / .E#t       ;K
E##;       tEE  Ej   t#      #;    ;tE.   f  \ ;  /  tt#f       f#E
EE##f      fEE  E#, :KW,      L     )##\   )  \/  ; .E#;      .EEf`
E#fE#f     t#f  E#t  ,#W:   ,KG  ,tEEE#EE#tt\   ,/ /E#/     iWE;
fEt D#G    fEE  fEt   ;#W. jEi  <tKEt#E###Ett;  \ )EE;    L##Lffi
E#t  E#E.  t#E  E#t   `i#KED.       )``/     ;  ,t#t    tLLG##L`
E#f   t#K: tEf  Eft      L#W. .__.--'  /       \,E#E##t--.  ,W#i`
fEt    ;EW,t#E  E#t   .EKj#K."--___   tt        .)#/##---"" E#E`
E#f     :K#D#E  fEt  iWf` i#K.   `/  tEE._..._/t#f_...   .D#j
E#t       .E##f E#t LK:    t#E   ;  / \EEf`   `"``   `( ,WK,
#;         `G#f fEt #       tDj ;  ; tE#EE\~—.  .—~`  EG.
#t           fE f#. #t       t; ;  / /E#/\##\   ; `\    #`
;#t          ;#  #   |      #; /` / .K#/ tEEE   \  ;  ;K
 t|          #`  f   `      |  ; / ,EE(.   fK#.   \_\  `K
  `          |t  `          `   `/  \K;    `t`     \`  `t
             t`                 `   `t          \
             `                      `            \
  '';

  environment.etc."fastfetch/config.jsonc".text = ''
{
  "$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",
  "logo": {
    "type": "file",
    "source": "/etc/fastfetch/logo.txt",
    "color": { "1": "magenta" },
    "padding": { "top": 1, "left": 2, "right": 4 }
  },
  "display": {
    "separator": " : "
  },
  "modules": [
    "title",
    "separator",
    { "type": "os", "key": "OS" },
    { "type": "kernel", "key": "Kernel" },
    { "type": "uptime", "key": "Uptime" },
    { "type": "packages", "key": "Paketler" },
    "break",
    { "type": "cpu", "key": "CPU" },
    "gpu",
    { "type": "memory", "key": "RAM" },
    { "type": "disk", "key": "Disk" },
    "break",
    { "type": "shell", "key": "Shell" },
    { "type": "wm", "key": "Masaustu" },
    { "type": "terminal", "key": "Terminal" },
    "break",
    { "type": "wifi", "key": "WiFi", "format": "{ssid} ({signal-quality}%)" },
    { "type": "localip", "key": "IP" },
    "break",
    "colors"
  ]
}
  '';
}

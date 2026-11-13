{
  description = "Taşınabilir, donanım-agnostik NixOS sistemi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-flatpak.url = "github:gmodena/nix-flatpak/?ref=latest";
    # follows: Serpantinum kendi nixpkgs'ini getirmesin (ikinci nixpkgs =
    # daha yavas degerlendirme ve diskte ikinci kopya kutuphaneler).
    serpantinum = {
      url = "github:ilyamiro/serpantinum";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    excalibur-wmi = {
      url = "github:thekayrasari/excalibur";
      flake = false;
    };
    # GRUB temasi (modules/desktop/grub-theme.nix). flake = false: upstream'in
    # kendi flake'i ayri bir nixpkgs getiriyor ve NixOS logosunu sandbox'ta
    # secemiyor (lsb_release); dosyalari kendimiz kopyaliyoruz.
    elegant-grub = {
      url = "github:vinceliuice/Elegant-grub2-themes";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, nix-flatpak, serpantinum, excalibur-wmi, elegant-grub, ... }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.portable = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit serpantinum excalibur-wmi elegant-grub; };
        modules = [
          nix-flatpak.nixosModules.nix-flatpak
          ./hosts/portable/default.nix
        ];
      };
    };
}

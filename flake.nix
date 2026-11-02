{
  description = "Taşınabilir, donanım-agnostik NixOS sistemi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-flatpak.url = "github:gmodena/nix-flatpak/?ref=latest";
    serpantinum.url = "github:ilyamiro/serpantinum";
    excalibur-wmi = {
      url = "github:thekayrasari/excalibur";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, nix-flatpak, serpantinum, excalibur-wmi, ... }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.portable = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit serpantinum excalibur-wmi; };
        modules = [
          nix-flatpak.nixosModules.nix-flatpak
          ./hosts/portable/default.nix
        ];
      };
    };
}

{
  description = "Taşınabilir, donanım-agnostik NixOS sistemi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-flatpak.url = "github:gmodena/nix-flatpak/?ref=latest";
  };

  outputs = { self, nixpkgs, nix-flatpak, ... }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.portable = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          nix-flatpak.nixosModules.nix-flatpak
          ./hosts/portable/default.nix
        ];
      };
    };
}

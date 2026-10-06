{
  description = "Taşınabilir, donanım-agnostik NixOS sistemi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-flatpak.url = "github:gmodena/nix-flatpak/?ref=latest";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ { self, nixpkgs, nix-flatpak, home-manager, ... }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.portable = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit inputs; };
        modules = [
          nix-flatpak.nixosModules.nix-flatpak
          ./hosts/portable/default.nix
        ];
      };
    };
}

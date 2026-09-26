{
  description = "Taşınabilir, donanım-agnostik NixOS sistemi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.portable = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          ./hosts/portable/default.nix
        ];
      };
    };
}

# PRIME girdisinin PCI adresleri: SADECE bu laptop (Casper Excalibur).
# Iki yerde kullanilir:
#   prime.nix        PRIME offload ayari
#   nerv/default.nix NERV, makinedeki kartlar bu adreslerde mi diye bakar;
#                    degilse PRIME yerine NVIDIA girdisini onerir.
{
  intel = "PCI:0@0:2:0";
  nvidia = "PCI:1@0:0:0";
}

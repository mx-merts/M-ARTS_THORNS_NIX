{ config, lib, pkgs, ... }:
{
  # AMAC: AppArmor kernel katmani acik ama COMPLAIN MODE'da - yani sadece
  # journal'a "bu uygulama boyle bir sey yapti" diye log yazar, HICBIR SEYI
  # ENGELLEMEZ. NixOS'ta enforce-mode profiller SDDM gibi servisleri
  # kilitleme riski tasidigi icin bilincli olarak bu seviyede tutuluyor.
  security.apparmor.enable = true;
}

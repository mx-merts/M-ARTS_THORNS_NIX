{ config, lib, pkgs, ... }:
{
  # AMAC: Sicaklik/fan okuma. Donanim-agnostik: eslesen cip yoksa modul
  # sessizce ENODEV verir, zararsiz.

  environment.systemPackages = with pkgs; [ lm_sensors ];

  boot.kernelModules = [
    "coretemp"   # Intel CPU sicakligi
    "k10temp"    # AMD CPU sicakligi
    "nct6775"    # Nuvoton Super I/O (ASUS/MSI/ASRock masaustu fanlari)
    "drivetemp"  # SATA/NVMe disk sicakligi
  ];

  # BIOS/ACPI'nin rezerve ettigi Super I/O portlarina surucunun erismesine izin verir.
  boot.kernelParams = [ "acpi_enforce_resources=lax" ];
}

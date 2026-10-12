#!/usr/bin/env bash
# /etc/nixos/rebuild.sh — Akilli rebuild modulu
# Akis: git add -A -> status + onay -> dry rebuild -> onay -> switch -> onay -> push -> ozet
# Kurallar:
#   - Yeni dosyalar otomatik git'e eklenir (flake "does not exist" tuzagi kapanir)
#   - Her asamanin ciktisi tam haliyle terminalde gorunur
#   - Dry basarisizsa switch calismaz, push yapilmaz
#   - Switch basarisizsa sayac harcanmaz, push yapilmaz
#   - Tum sorularda Enter = Hayir (guvenli varsayilan)
# NOT: set -e bilerek YOK — hatalari yakalayip ozet gostermek icin

COUNT_FILE=/etc/nixos/.rebuild-count

ONAY="verilmedi"
DRY_RESULT="yapilmadi"
REBUILD_RESULT="yapilmadi"
PUSH_RESULT="yapilmadi"
GRUB_LABEL="-"

ozet() {
  echo ""
  echo "================= REBUILD OZETI ================="
  printf "%-16s: %s\n" "git add -A" "tamam"
  printf "%-16s: %s\n" "onay" "$ONAY"
  printf "%-16s: %s\n" "dry rebuild" "$DRY_RESULT"
  printf "%-16s: %s\n" "gercek rebuild" "$REBUILD_RESULT"
  printf "%-16s: %s\n" "GRUB etiketi" "$GRUB_LABEL"
  printf "%-16s: %s\n" "git push" "$PUSH_RESULT"
  echo "================================================="
}

ask() {
  # 0 = evet, 1 = hayir. Enter = hayir (guvenli varsayilan)
  local cevap
  read -r -p "$1 [e/H]: " cevap
  case "$cevap" in
    e|E|evet|Evet|EVET) return 0 ;;
    *) return 1 ;;
  esac
}

echo ""
echo "=============== 1/5 GIT HAZIRLIK ==============="
git -C /etc/nixos add -A
echo "--- git status ---"
git -C /etc/nixos status
echo ""
echo "--- staged degisiklik istatistigi ---"
git -C /etc/nixos diff --cached --stat
echo ""

if ! ask "Bu degisikliklerle devam edilsin mi?"; then
  ONAY="REDDEDILDI — islem iptal"
  ozet
  exit 0
fi
ONAY="verildi"

# GRUB etiketi icin aday sayac (sadece switch basarili olursa dosyaya yazilir)
N=$(( $(cat "$COUNT_FILE" 2>/dev/null || echo 0) + 1 ))

echo ""
echo "=============== 2/5 DRY REBUILD ================"
echo "dry-activate: config tam derlenir + degisiklik raporu gosterilir."
echo "HICBIR SEY uygulanmaz: sayac, GRUB, calisan sistem degismez."
echo "Aday GRUB etiketi: T-${N} (dry ve switch ayni etiketle degerlendirilir)"
echo ""
if NIXOS_LABEL_VERSION=":-:-:T-${N}:-:-:" nixos-rebuild dry-activate --flake /etc/nixos#portable --impure "$@"; then
  DRY_RESULT="BASARILI"
  echo ""
  echo ">>> Dry rebuild BASARILI."
else
  DRY_RESULT="BASARISIZ"
  echo ""
  echo ">>> Dry rebuild BASARISIZ! Hatayi yukaridan incele. Sistem degismedi."
  ozet
  exit 1
fi

if ! ask "Gercek rebuild (switch) yapilsin mi?"; then
  REBUILD_RESULT="atlandi (kullanici)"
  ozet
  exit 0
fi

echo ""
echo "============ 3/5 GERCEK REBUILD + SWITCH ============"
echo ""
if NIXOS_LABEL_VERSION=":-:-:T-${N}:-:-:" nixos-rebuild switch --flake /etc/nixos#portable --impure "$@"; then
  REBUILD_RESULT="BASARILI"
  echo "$N" > "$COUNT_FILE"
  GRUB_LABEL="T-${N}"
  echo ""
  echo ">>> Switch BASARILI. Sayac yazildi: T-${N}"
else
  REBUILD_RESULT="BASARISIZ"
  GRUB_LABEL="T-${N} (harcanmadi)"
  echo ""
  echo ">>> Switch BASARISIZ! Sayac yazilmadi (T-${N} harcanmadi)."
  echo ">>> Sistem onceki config ile calismaya devam ediyor. Push yapilmaz."
  ozet
  exit 1
fi

if ! ask "GitHub'a push edilsin mi?"; then
  PUSH_RESULT="atlandi (kullanici)"
  ozet
  exit 0
fi

echo ""
echo "=============== 4/5 GIT PUSH ================"
read -r -p "Commit mesaji (bos = otomatik tarih-saat): " COMMIT_MSG
echo ""
if [ -n "$COMMIT_MSG" ]; then
  if /etc/nixos/push.sh "$COMMIT_MSG"; then
    PUSH_RESULT="BASARILI"
  else
    PUSH_RESULT="BASARISIZ"
  fi
else
  if /etc/nixos/push.sh; then
    PUSH_RESULT="BASARILI"
  else
    PUSH_RESULT="BASARISIZ"
  fi
fi

echo ""
echo "================= 5/5 BITTI ================="
ozet

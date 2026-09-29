#!/usr/bin/env bash
# /etc/nixos/push.sh - Git add + commit + push (tek komut)
# Kullanim:
#   sudo /etc/nixos/push.sh                  -> otomatik mesaj (tarih-saat)
#   sudo /etc/nixos/push.sh "feat: yeni mod" -> ozel mesaj
set -e

cd /etc/nixos

MSG="${1:-"update: $(date '+%Y-%m-%d %H:%M')"}"

# Degisiklik var mi?
if [ -z "$(git status --porcelain)" ]; then
  echo "Degisiklik yok, push atlandi."
  exit 0
fi

echo "=== Degisen dosyalar ==="
git status --short
echo ""

git add -A
git commit -m "$MSG"
git push

echo ""
echo "Push tamamlandi: $MSG"

#!/usr/bin/env bash
# /etc/nixos/push.sh - Git add + commit + push (single command)
# Usage:
#   sudo /etc/nixos/push.sh                  -> automatic message (date-time)
#   sudo /etc/nixos/push.sh "feat: new mod"  -> custom message
set -e

cd /etc/nixos

MSG="${1:-"update: $(date '+%Y-%m-%d %H:%M')"}"

# Any changes?
if [ -z "$(git status --porcelain)" ]; then
  echo "No changes, push skipped."
  exit 0
fi

echo "=== Changed files ==="
git status --short
echo ""

git add -A
git commit -m "$MSG"
git push

echo ""
echo "Push completed: $MSG"

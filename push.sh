#!/usr/bin/env bash
# /etc/nixos/push.sh - Git add + commit + push (single command)
# Usage:
#   sudo /etc/nixos/push.sh                  -> automatic message (date-time)
#   sudo /etc/nixos/push.sh "feat: new mod"  -> custom message
set -e

cd /etc/nixos

MSG="${1:-"update: $(date '+%Y-%m-%d %H:%M')"}"

# Working-tree changes?
DIRTY=$(git status --porcelain | wc -l)
# Unpushed commits? (e.g. a previous push was rejected by the remote)
UNPUSHED=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo 0)

if [ "$DIRTY" -eq 0 ] && [ "$UNPUSHED" -eq 0 ]; then
  echo "No changes, push skipped."
  exit 0
fi

if [ "$DIRTY" -gt 0 ]; then
  echo "=== Changed files ==="
  git status --short
  echo ""
  git add -A
  git commit -m "$MSG"
fi

if [ "$UNPUSHED" -gt 0 ]; then
  echo "=== $UNPUSHED unpushed commit(s) found, pushing them too ==="
fi

git push

echo ""
echo "Push completed."

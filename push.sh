#!/usr/bin/env bash
# /etc/nixos/push.sh - Git add + commit + push (single command)
# Usage:
#   sudo /etc/nixos/push.sh                  -> automatic message (date-time)
#   sudo /etc/nixos/push.sh "feat: new mod"  -> custom message
set -e

cd /etc/nixos

MSG="${1:-"update: $(date '+%Y-%m-%d %H:%M')"}"

# Working-tree changes? Commit them first.
if [ -n "$(git status --porcelain)" ]; then
  echo "=== Changed files ==="
  git status --short
  echo ""
  git add -A
  git commit -m "$MSG"
fi

# Count AFTER committing, so this number is always accurate.
UNPUSHED=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo 0)

if [ "$UNPUSHED" -eq 0 ]; then
  echo "No changes, push skipped."
  exit 0
fi

echo "=== $UNPUSHED unpushed commit(s), pushing ==="
git push

echo ""
echo "Push completed."

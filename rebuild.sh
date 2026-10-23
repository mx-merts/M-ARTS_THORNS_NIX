#!/usr/bin/env bash
# /etc/nixos/rebuild.sh — Smart rebuild module
# Flow: git add -A -> status + confirm -> dry rebuild -> confirm -> switch -> confirm -> push -> summary
# Rules:
#   - New files are staged automatically (avoids the flake "does not exist" trap)
#   - Full output of every stage is shown in the terminal
#   - If dry fails: no switch, no push
#   - If switch fails: counter is not consumed, no push
#   - Enter = No on all prompts (safe default)
# NOTE: set -e is deliberately OMITTED — errors are caught so the summary can still be shown

COUNT_FILE=/etc/nixos/.rebuild-count

CONFIRM="not given"
DRY_RESULT="not run"
REBUILD_RESULT="not run"
PUSH_RESULT="not run"
GRUB_LABEL="-"
AIDE_RESULT="not run"

summary() {
  echo ""
  echo "================= REBUILD SUMMARY ================="
  printf "%-16s: %s\n" "git add -A" "done"
  printf "%-16s: %s\n" "confirmation" "$CONFIRM"
  printf "%-16s: %s\n" "dry rebuild" "$DRY_RESULT"
  printf "%-16s: %s\n" "real rebuild" "$REBUILD_RESULT"
  printf "%-16s: %s\n" "GRUB label" "$GRUB_LABEL"
  printf "%-16s: %s\n" "AIDE (boot)" "$AIDE_RESULT"
  printf "%-16s: %s\n" "git push" "$PUSH_RESULT"
  echo "==================================================="
}

ask() {
  # 0 = yes, 1 = no. Enter = no (safe default)
  local answer
  read -r -p "$1 [y/N]: " answer
  case "$answer" in
    y|Y|yes|Yes|YES) return 0 ;;
    *) return 1 ;;
  esac
}

echo ""
echo "=============== 1/5 GIT PREP ==============="
git -C /etc/nixos add -A
echo "--- git status ---"
git -C /etc/nixos status
echo ""
echo "--- staged changes stat ---"
git -C /etc/nixos diff --cached --stat
echo ""

if ! ask "Continue with these changes?"; then
  CONFIRM="DENIED — aborted"
  summary
  exit 0
fi
CONFIRM="given"

# AIDE: compare /boot with its baseline BEFORE touching it. A rebuild changes
# /boot legitimately, so anything that differs now happened outside a rebuild.
AIDE_CHECK="skipped"
if command -v thorns-aide >/dev/null 2>&1; then
  echo ""
  echo "=============== AIDE PRE-CHECK (/boot) ==============="
  if thorns-aide pre-rebuild --alert; then
    AIDE_CHECK="clean"
  else
    AIDE_CHECK="DIFFERS"
    if ! ask "AIDE: /boot changed outside a rebuild (see above). Continue anyway?"; then
      CONFIRM="aborted (AIDE /boot differs)"
      AIDE_RESULT="DIFFERS - aborted"
      summary
      exit 0
    fi
  fi
fi

# Candidate GRUB label (written to file only if the switch succeeds)
N=$(( $(cat "$COUNT_FILE" 2>/dev/null || echo 0) + 1 ))

echo ""
echo "=============== 2/5 DRY REBUILD ================"
echo "dry-activate: fully builds the config + prints a change report."
echo "NOTHING is applied: counter, GRUB and the running system stay untouched."
echo "Candidate GRUB label: T-${N} (dry and switch are evaluated with the same label)"
echo ""
if NIXOS_LABEL_VERSION=":-:-:T-${N}:-:-:" nixos-rebuild dry-activate --flake /etc/nixos#portable --impure "$@"; then
  DRY_RESULT="SUCCESS"
  echo ""
  echo ">>> Dry rebuild PASSED."
else
  DRY_RESULT="FAILED"
  echo ""
  echo ">>> Dry rebuild FAILED! Inspect the error above. System unchanged."
  summary
  exit 1
fi

if ! ask "Proceed with the real rebuild (switch)?"; then
  REBUILD_RESULT="skipped (user)"
  summary
  exit 0
fi

echo ""
echo "============ 3/5 REAL REBUILD + SWITCH ============"
echo ""
if NIXOS_LABEL_VERSION=":-:-:T-${N}:-:-:" nixos-rebuild switch --flake /etc/nixos#portable --impure "$@"; then
  REBUILD_RESULT="SUCCESS"
  echo "$N" > "$COUNT_FILE"
  if command -v thorns-aide >/dev/null 2>&1; then
    if thorns-aide post-rebuild; then
      AIDE_RESULT="pre-check ${AIDE_CHECK}, baseline refreshed"
    else
      AIDE_RESULT="pre-check ${AIDE_CHECK}, baseline refresh FAILED"
    fi
  fi
  GRUB_LABEL="T-${N}"
  echo ""
  echo ">>> Switch PASSED. Counter written: T-${N}"
else
  REBUILD_RESULT="FAILED"
  GRUB_LABEL="T-${N} (not consumed)"
  echo ""
  echo ">>> Switch FAILED! Counter not written (T-${N} not consumed)."
  echo ">>> System keeps running the previous config. No push will be made."
  summary
  exit 1
fi

if ! ask "Push to GitHub?"; then
  PUSH_RESULT="skipped (user)"
  summary
  exit 0
fi

echo ""
echo "=============== 4/5 GIT PUSH ================"
read -r -p "Commit message (empty = automatic date-time): " COMMIT_MSG
echo ""
if [ -n "$COMMIT_MSG" ]; then
  if /etc/nixos/push.sh "$COMMIT_MSG"; then
    PUSH_RESULT="SUCCESS"
  else
    PUSH_RESULT="FAILED"
  fi
else
  if /etc/nixos/push.sh; then
    PUSH_RESULT="SUCCESS"
  else
    PUSH_RESULT="FAILED"
  fi
fi

echo ""
echo "================= 5/5 DONE ================="
summary

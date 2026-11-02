#!/usr/bin/env bash
# /etc/nixos/rebuild.sh — Smart rebuild module
# Flow: git add -A -> status + confirm -> [lock] -> dry rebuild -> confirm
#       -> pick switch target -> rebuild -> confirm -> push -> summary
# Rules:
#   - New files are staged automatically (avoids the flake "does not exist" trap)
#   - Full output of every stage is shown in the terminal
#   - If dry fails: no switch, no push
#   - If the rebuild fails: counter is not consumed, no push
#   - Enter = No on all y/N prompts (safe default); Enter on the target menu
#     = stay in the config you booted (safe default)
#   - The lock (/run/thorns-rebuild.lock) is shared with thorns-upgrade:
#     only one of them may touch /boot or flake.lock at a time. Held until exit.
#   - GRUB is ALWAYS installed from the BASE config ("nixos-rebuild boot"), so
#     the main menu keeps every entry (NixOS, PRIME, NVIDIA-DESKTOP). The
#     chosen target is then activated live with "switch-to-configuration test",
#     which never touches the boot loader.
#     NEVER use "switch --specialisation": it installs GRUB from the
#     specialisation's point of view, so the main menu loses the PRIME /
#     NVIDIA-DESKTOP entries and plain "NixOS" boots the specialisation
#     (happened 2026-11-02 with T-97/T-98).
# NOTE: set -e is deliberately OMITTED — errors are caught so the summary can still be shown

COUNT_FILE=/etc/nixos/.rebuild-count
LOCK_FILE=/run/thorns-rebuild.lock

CONFIRM="not given"
DRY_RESULT="not run"
REBUILD_RESULT="not run"
PUSH_RESULT="not run"
GRUB_LABEL="-"
AIDE_RESULT="not run"
LOCK_RESULT="not acquired"
SPEC_RESULT="-"
TARGET_RESULT="-"

summary() {
  echo ""
  echo "================= REBUILD SUMMARY ================="
  printf "%-16s: %s\n" "git add -A" "done"
  printf "%-16s: %s\n" "confirmation" "$CONFIRM"
  printf "%-16s: %s\n" "rebuild lock" "$LOCK_RESULT"
  printf "%-16s: %s\n" "booted" "$SPEC_RESULT"
  printf "%-16s: %s\n" "switch target" "$TARGET_RESULT"
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

# Which specialisation was this machine booted into? Prints its name, or
# nothing for the base config. Compares /run/booted-system against the
# specialisation toplevels of every system generation still on disk.
detect_specialisation() {
  local booted link
  booted="$(readlink -f /run/booted-system 2>/dev/null)" || return 0
  [ -n "$booted" ] || return 0
  for link in /nix/var/nix/profiles/system-*-link/specialisation/*; do
    [ -e "$link" ] || continue
    if [ "$(readlink -f "$link")" = "$booted" ]; then
      basename "$link"
      return 0
    fi
  done
}

# Interactive target menu. Sets TARGET and TARGET_LABEL:
#   TARGET=""        -> base config (DESK), activated live
#   TARGET="<name>"  -> that specialisation, activated live
#   TARGET="@boot"   -> GRUB only, running system untouched
#   TARGET="@cancel" -> abort
# The list of specialisations is read from the current system profile, so a
# new specialisation shows up here automatically.
choose_target() {
  local link i choice mark default=1
  TARGET_NAMES=("")
  TARGET_LABELS=("DESK (base config)")
  for link in /nix/var/nix/profiles/system/specialisation/*; do
    [ -e "$link" ] || continue
    TARGET_NAMES+=("$(basename "$link")")
    TARGET_LABELS+=("$(basename "$link")")
  done
  TARGET_NAMES+=("@boot")
  TARGET_LABELS+=("GRUB only: change nothing now, apply at next reboot")

  for i in "${!TARGET_NAMES[@]}"; do
    if [ "${TARGET_NAMES[$i]}" = "$SPEC" ]; then default=$((i + 1)); fi
  done

  while true; do
    echo ""
    echo "=============== SWITCH TARGET ==============="
    echo "GRUB is always installed from the base config (all entries stay in the menu)."
    echo "Choose what the RUNNING session switches to:"
    for i in "${!TARGET_NAMES[@]}"; do
      mark=""
      if [ $((i + 1)) -eq "$default" ]; then mark="   <- booted (Enter)"; fi
      printf "  %d) %s%s\n" $((i + 1)) "${TARGET_LABELS[$i]}" "$mark"
    done
    echo "  0) cancel"
    read -r -p "Target [${default}]: " choice
    [ -z "$choice" ] && choice="$default"
    case "$choice" in
      0) TARGET="@cancel"; TARGET_LABEL="cancel"; return 0 ;;
      *[!0-9]*) echo "Invalid choice: $choice"; continue ;;
    esac
    if [ "$choice" -lt 1 ] || [ "$choice" -gt "${#TARGET_NAMES[@]}" ]; then
      echo "Invalid choice: $choice"
      continue
    fi
    TARGET="${TARGET_NAMES[$((choice - 1))]}"
    TARGET_LABEL="${TARGET_LABELS[$((choice - 1))]}"
    if [ "$TARGET" != "@boot" ] && [ "$TARGET" != "$SPEC" ]; then
      echo ""
      echo "WARNING: you booted '${SPEC:-DESK}' but picked '${TARGET_LABEL}'."
      echo "         GPU drivers (NVIDIA / PRIME) cannot be swapped cleanly in a"
      echo "         running session; the change is only complete after a reboot."
      echo "         Safer: pick 'GRUB only' and choose the entry in GRUB."
      if ! ask "Switch the running session to '${TARGET_LABEL}' anyway?"; then
        continue
      fi
    fi
    return 0
  done
}

# The real rebuild.
#   return 0: built + GRUB installed (+ live activation done, unless GRUB only)
#   return 1: build or GRUB install failed -> nothing changed
#   return 2: GRUB installed (new generation exists) but live activation
#             failed -> a reboot applies it
do_rebuild() {
  # 1) Build + set the profile + install GRUB from the BASE config.
  NIXOS_LABEL_VERSION=":-:-:T-${N}:-:-:" nixos-rebuild boot --flake /etc/nixos#portable --impure "$@" || return 1

  [ "$TARGET" = "@boot" ] && return 0

  # 2) Activate the chosen config of the NEW generation, live only.
  local top target
  top="$(readlink -f /nix/var/nix/profiles/system)"
  if [ -z "$TARGET" ]; then
    target="$top/bin/switch-to-configuration"
  else
    target="$top/specialisation/${TARGET}/bin/switch-to-configuration"
  fi
  if [ ! -x "$target" ]; then
    echo ">>> '${TARGET_LABEL}' not found in the new generation." >&2
    return 2
  fi
  echo ""
  echo ">>> GRUB installed from the base config. Activating '${TARGET_LABEL}' in the running session (test)..."
  # Same isolation nixos-rebuild uses: survives if the terminal or session restarts.
  systemd-run -E LOCALE_ARCHIVE --collect --no-ask-password --pipe --quiet \
    --service-type=exec --unit=nixos-rebuild-switch-to-configuration \
    "$target" test || return 2
  return 0
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

# ── Rebuild lock ─────────────────────────────────────────────────────────────
# Shared with thorns-upgrade (modules/core/auto-upgrade.nix). Guarantees that
# nothing else mutates /boot or flake.lock between the AIDE pre-check and the
# post-rebuild re-baseline (the 2026-10-25 race). /run is tmpfs: no stale locks.
exec 9>"$LOCK_FILE"
if flock -n 9; then
  LOCK_RESULT="acquired"
else
  echo ""
  echo "=============== REBUILD LOCK BUSY ==============="
  echo "thorns-upgrade or another rebuild is running right now."
  if ask "Wait up to 10 minutes for the lock?"; then
    if flock -w 600 9; then
      LOCK_RESULT="acquired after wait"
    else
      LOCK_RESULT="timeout (10 min)"
      REBUILD_RESULT="skipped (lock timeout)"
      summary
      exit 1
    fi
  else
    LOCK_RESULT="busy — aborted"
    REBUILD_RESULT="skipped (user)"
    summary
    exit 0
  fi
fi

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

# ── Booted config ────────────────────────────────────────────────────────────
SPEC="$(detect_specialisation)"
SPEC_RESULT="${SPEC:-DESK (base)}"
echo ""
echo "Booted: ${SPEC:-DESK (base)}"

# Candidate GRUB label (written to file only if the rebuild succeeds)
N=$(( $(cat "$COUNT_FILE" 2>/dev/null || echo 0) + 1 ))

echo ""
echo "=============== 2/5 DRY REBUILD ================"
echo "dry-activate: fully builds the config + prints a change report."
echo "NOTHING is applied: counter, GRUB and the running system stay untouched."
echo "Candidate GRUB label: T-${N} (dry and real rebuild use the same label)"
echo "NOTE: the dry report is computed for the BASE config (DESK). The real"
echo "      rebuild installs GRUB from it and then activates the target you pick."
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

if ! ask "Proceed with the real rebuild?"; then
  REBUILD_RESULT="skipped (user)"
  summary
  exit 0
fi

choose_target
if [ "$TARGET" = "@cancel" ]; then
  TARGET_RESULT="cancelled"
  REBUILD_RESULT="skipped (user)"
  summary
  exit 0
fi
case "$TARGET" in
  "")     TARGET_RESULT="DESK (live)" ;;
  "@boot") TARGET_RESULT="GRUB only (apply at next reboot)" ;;
  *)      TARGET_RESULT="${TARGET} (live)" ;;
esac

echo ""
echo "============ 3/5 REAL REBUILD + SWITCH ============"
echo ""
do_rebuild "$@"
RC=$?

if [ "$RC" -eq 1 ]; then
  REBUILD_RESULT="FAILED"
  GRUB_LABEL="T-${N} (not consumed)"
  echo ""
  echo ">>> Rebuild FAILED! Counter not written (T-${N} not consumed)."
  echo ">>> System keeps running the previous config. No push will be made."
  summary
  exit 1
fi

# RC 0 or 2: a new generation with label T-N is in GRUB -> consume the counter.
echo "$N" > "$COUNT_FILE"
GRUB_LABEL="T-${N}"
if command -v thorns-aide >/dev/null 2>&1; then
  if thorns-aide post-rebuild; then
    AIDE_RESULT="pre-check ${AIDE_CHECK}, baseline refreshed"
  else
    AIDE_RESULT="pre-check ${AIDE_CHECK}, baseline refresh FAILED"
  fi
fi

if [ "$RC" -eq 2 ]; then
  REBUILD_RESULT="PARTIAL: in GRUB, live activation FAILED"
  echo ""
  echo ">>> T-${N} is installed in GRUB, but activating '${TARGET_LABEL}' live FAILED."
  echo ">>> Reboot and pick it in GRUB to apply. No push will be made."
  summary
  exit 1
fi

REBUILD_RESULT="SUCCESS"
echo ""
if [ "$TARGET" = "@boot" ]; then
  echo ">>> T-${N} installed in GRUB. Running system unchanged: reboot to apply."
else
  echo ">>> Rebuild PASSED, running session is now '${TARGET_LABEL}'. Counter written: T-${N}"
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

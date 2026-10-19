{ config, lib, pkgs, ... }:
let
  cscan = pkgs.writeShellApplication {
    name = "CSCAN";
    runtimeInputs = with pkgs; [ clamav findutils coreutils gnugrep ];
    text = ''
      TARGET="''${1:-.}"
      CHUNK="''${CSCAN_CHUNK:-300}"

      if [ ! -e "$TARGET" ]; then
        echo "CSCAN: not found: $TARGET" >&2
        exit 1
      fi
      TARGET="$(realpath "$TARGET")"

      # clamd.conf'taki ExcludePath ile ayni mantik
      EXCLUDE='/\.local/share/Steam/|/\.var/|/Games/|/\.cache/|/\.local/share/Trash/|/node_modules/|/\.git/objects/'

      TMP="$(mktemp -d)"
      trap 'rm -rf "$TMP"' EXIT

      echo "Dosyalar listeleniyor: $TARGET"
      find "$TARGET" -type f -print 2>/dev/null | grep -Ev "$EXCLUDE" > "$TMP/all" || true

      TOTAL="$(wc -l < "$TMP/all")"
      if [ "$TOTAL" -eq 0 ]; then
        echo "Taranacak dosya yok."
        exit 0
      fi
      echo "Toplam $TOTAL dosya taranacak."

      split -l "$CHUNK" "$TMP/all" "$TMP/chunk."

      DONE=0
      INFECTED=0
      ERRORS=0
      START="$(date +%s)"

      for list in "$TMP"/chunk.*; do
        N="$(wc -l < "$list")"
        RC=0
        OUT="$(clamdscan --fdpass --multiscan --infected --no-summary \
                --file-list="$list" 2>&1)" || RC=$?

        if [ "$RC" -eq 1 ]; then
          FOUND="$(printf '%s\n' "$OUT" | grep -c 'FOUND' || true)"
          INFECTED=$((INFECTED + FOUND))
          printf '\r\033[K'
          printf '%s\n' "$OUT" | grep 'FOUND' || true
        elif [ "$RC" -ge 2 ]; then
          ERRORS=$((ERRORS + 1))
          printf '\r\033[K'
          printf '%s\n' "$OUT" | head -n 3 >&2
        fi

        DONE=$((DONE + N))
        PCT=$((DONE * 100 / TOTAL))
        FILLED=$((PCT * 30 / 100))
        BAR="$(printf '%*s' "$FILLED" "" | tr ' ' '#')"
        ELAPSED=$(( $(date +%s) - START ))
        printf '\r\033[K[%-30s] %3d%%  %d/%d  tehdit: %d  %ds' \
          "$BAR" "$PCT" "$DONE" "$TOTAL" "$INFECTED" "$ELAPSED"
      done

      echo ""
      echo "=== Bitti: $TOTAL dosya, $INFECTED tehdit, $ERRORS hatali grup, $(( $(date +%s) - START )) sn ==="

      if [ "$INFECTED" -gt 0 ]; then exit 1; fi
      if [ "$ERRORS" -gt 0 ]; then exit 2; fi
    '';
  };
in
{
  environment.systemPackages = [ cscan ];
}

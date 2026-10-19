{ config, lib, pkgs, ... }:
let
  cscan = pkgs.writeShellApplication {
    name = "CSCAN";
    excludeShellChecks = [ "SC2016" "SC2002" ];
    runtimeInputs = with pkgs; [ clamav findutils coreutils gnugrep ];
    text = ''
      TARGET="''${1:-.}"
      CHUNK="''${CSCAN_CHUNK:-200}"
      JOBS="''${CSCAN_JOBS:-8}"

      if [ ! -e "$TARGET" ]; then
        echo "CSCAN: not found: $TARGET" >&2
        exit 1
      fi
      TARGET="$(realpath "$TARGET")"

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
      echo "Toplam $TOTAL dosya, $JOBS paralel is parcacigi."

      split -l "$CHUNK" -d -a 5 "$TMP/all" "$TMP/chunk."
      START="$(date +%s)"

      # Gruplari JOBS adet paralel clamdscan sureciyle tara
      (
        find "$TMP" -maxdepth 1 -name 'chunk.[0-9]*' ! -name '*.out' ! -name '*.rc' -print0 |
          xargs -0 -P "$JOBS" -I{} sh -c \
            'clamdscan --fdpass --infected --no-summary --file-list="$1" > "$1.out" 2>&1; echo $? > "$1.rc"' _ {}
      ) &
      BG=$!

      progress() {
        DONE=0
        for rc in "$TMP"/chunk.*.rc; do
          [ -e "$rc" ] || continue
          DONE=$((DONE + $(wc -l < "''${rc%.rc}")))
        done
        INFECTED="$(cat "$TMP"/chunk.*.out 2>/dev/null | grep -c 'FOUND' || true)"
        PCT=$((DONE * 100 / TOTAL))
        FILLED=$((PCT * 30 / 100))
        BAR="$(printf '%*s' "$FILLED" "" | tr ' ' '#')"
        printf '\r\033[K[%-30s] %3d%%  %d/%d  tehdit: %s  %ds' \
          "$BAR" "$PCT" "$DONE" "$TOTAL" "$INFECTED" "$(( $(date +%s) - START ))"
      }

      while kill -0 "$BG" 2>/dev/null; do
        progress
        sleep 0.5
      done
      wait "$BG" || true
      progress
      echo ""

      ERRORS=0
      for rc in "$TMP"/chunk.*.rc; do
        [ -e "$rc" ] || continue
        if [ "$(cat "$rc")" -ge 2 ]; then
          ERRORS=$((ERRORS + 1))
          head -n 3 "''${rc%.rc}.out" >&2
        fi
      done

      echo ""
      cat "$TMP"/chunk.*.out 2>/dev/null | grep 'FOUND' || echo "Tehdit bulunmadi."
      echo "=== Bitti: $TOTAL dosya, $ERRORS hatali grup, $(( $(date +%s) - START )) sn ==="

      if [ "$ERRORS" -gt 0 ]; then exit 2; fi
      if cat "$TMP"/chunk.*.out 2>/dev/null | grep -q 'FOUND'; then exit 1; fi
    '';
  };
in
{
  environment.systemPackages = [ cscan ];
}

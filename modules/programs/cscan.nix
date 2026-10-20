{ config, lib, pkgs, ... }:
let
  cscan = pkgs.writeShellApplication {
    name = "CSCAN";
    excludeShellChecks = [ "SC2016" "SC2002" ];
    runtimeInputs = with pkgs; [ clamav findutils coreutils gnugrep ];
    text = ''
      CHUNK="''${CSCAN_CHUNK:-200}"
      JOBS="''${CSCAN_JOBS:-8}"

      usage() {
        echo "kullanim: sudo CSCAN <mod>"
        echo "  -ROT    root (/) taranir, /home haric"
        echo "  -BOT    /boot taranir"
        echo "  -HOME   /home taranir"
        echo "  -RAW    hepsi: / + /home + /boot + harici diskler"
        echo "  <yol>   verilen yol(lar) taranir"
      }

      if [ "$(id -u)" -ne 0 ]; then
        echo "CSCAN: root gerekli, 'sudo CSCAN ...' ile calistir." >&2
        exit 1
      fi

      if [ $# -eq 0 ]; then
        usage
        exit 1
      fi

      MODE="''${1^^}"
      MODE="''${MODE#-}"
      MODE="''${MODE#-}"

      TMP="$(mktemp -d)"
      trap 'rm -rf "$TMP"' EXIT

      media_dirs() {
        for m in /run/media/*/*; do
          if [ -d "$m" ]; then echo "$m"; fi
        done
      }

      case "$MODE" in
        H|HELP)
          usage
          exit 0
          ;;
        ROT)
          echo "Mod: ROT (/ , /home haric)"
          find / -xdev -path /home -prune -o -type f -print > "$TMP/all" 2>/dev/null || true
          ;;
        BOT)
          echo "Mod: BOT (/boot)"
          find /boot -xdev -type f -print > "$TMP/all" 2>/dev/null || true
          ;;
        HOME)
          echo "Mod: HOME (/home)"
          find /home -xdev -type f -print > "$TMP/all" 2>/dev/null || true
          ;;
        RAW)
          echo "Mod: RAW (/ + /home + /boot + harici diskler)"
          {
            find / -xdev -type f -print 2>/dev/null || true
            find /boot -xdev -type f -print 2>/dev/null || true
            media_dirs | while read -r m; do
              find "$m" -xdev -type f -print 2>/dev/null || true
            done
          } > "$TMP/all"
          ;;
        *)
          TARGETS=()
          for t in "$@"; do
            if [ ! -e "$t" ]; then
              echo "CSCAN: bulunamadi: $t" >&2
              usage
              exit 1
            fi
            TARGETS+=("$(realpath "$t")")
          done
          echo "Mod: ozel yol (''${TARGETS[*]})"
          find "''${TARGETS[@]}" -xdev -type f -print > "$TMP/all" 2>/dev/null || true
          ;;
      esac

      TOTAL="$(wc -l < "$TMP/all")"
      if [ "$TOTAL" -eq 0 ]; then
        echo "Taranacak dosya yok."
        exit 0
      fi
      echo "Toplam $TOTAL dosya, $JOBS paralel is parcacigi. Hicbir sey atlanmiyor."

      split -l "$CHUNK" -d -a 5 "$TMP/all" "$TMP/chunk."
      START="$(date +%s)"

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

      if cat "$TMP"/chunk.*.out 2>/dev/null | grep -q 'FOUND'; then exit 1; fi
      if [ "$ERRORS" -gt 0 ]; then exit 2; fi
    '';
  };
in
{
  environment.systemPackages = [ cscan ];
}

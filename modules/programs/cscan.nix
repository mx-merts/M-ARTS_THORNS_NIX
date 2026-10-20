{ config, lib, pkgs, ... }:
let
  cscan = pkgs.writeShellApplication {
    name = "CSCAN";
    excludeShellChecks = [ "SC2016" "SC2002" ];
    runtimeInputs = with pkgs; [ clamav findutils coreutils gnugrep gawk ];
    text = ''
      CHUNK="''${CSCAN_CHUNK:-200}"
      JOBS="''${CSCAN_JOBS:-8}"
      LOG=/var/log/cscan.log

      usage() {
        echo "kullanim: sudo CSCAN <mod> [--TERMINATE]"
        echo "  -ROT        root (/) taranir, /home haric"
        echo "  -BOT        /boot taranir"
        echo "  -HOME       /home taranir"
        echo "  -RAW        hepsi: / + /home + /boot + harici diskler"
        echo "  <yol>       verilen yol(lar) taranir"
        echo "  --TERMINATE bulunan tehditleri KALICI olarak siler"
      }

      if [ "$(id -u)" -ne 0 ]; then
        echo "CSCAN: root gerekli, 'sudo CSCAN ...' ile calistir." >&2
        exit 1
      fi

      TERMINATE=0
      ARGS=()
      for a in "$@"; do
        case "''${a^^}" in
          --TERMINATE|-TERMINATE) TERMINATE=1 ;;
          *) ARGS+=("$a") ;;
        esac
      done

      if [ ''${#ARGS[@]} -eq 0 ]; then
        usage
        exit 1
      fi
      set -- "''${ARGS[@]}"

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

      if [ "$TERMINATE" -eq 1 ]; then
        echo "UYARI: --TERMINATE aktif, bulunan tehditler tarama sonunda SILINECEK."
      fi

      TOTAL="$(wc -l < "$TMP/all")"
      if [ "$TOTAL" -eq 0 ]; then
        echo "Taranacak dosya yok."
        exit 0
      fi
      echo "Toplam $TOTAL dosya, $JOBS paralel is parcacigi. Hicbir sey atlanmiyor."

      split -l "$CHUNK" -d -a 5 "$TMP/all" "$TMP/chunk."
      START="$(date +%s)"
      START_DATE="$(date '+%Y:%m:%d %H:%M:%S')"

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
        INFECTED="$(cat "$TMP"/chunk.*.out 2>/dev/null | grep -c ' FOUND$' || true)"
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

      END_DATE="$(date '+%Y:%m:%d %H:%M:%S')"
      ELAPSED=$(( $(date +%s) - START ))

      ERRORS=0
      for rc in "$TMP"/chunk.*.rc; do
        [ -e "$rc" ] || continue
        if [ "$(cat "$rc")" -ge 2 ]; then
          ERRORS=$((ERRORS + 1))
        fi
      done

      cat "$TMP"/chunk.*.out 2>/dev/null | grep ' FOUND$' > "$TMP/found" || true
      cat "$TMP"/chunk.*.out 2>/dev/null | grep '^ERROR' > "$TMP/errlines" || true
      FOUND_COUNT="$(wc -l < "$TMP/found")"
      ERR_FILES="$(wc -l < "$TMP/errlines")"

      echo ""
      if [ "$FOUND_COUNT" -gt 0 ]; then
        echo "--- BULUNAN TEHDITLER ---"
        cat "$TMP/found"
        while IFS= read -r line; do
          printf '%s  %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$line" >> "$LOG"
        done < "$TMP/found"
        echo ""
      fi

      REMOVED=0
      REMOVE_FAILED=0
      if [ "$TERMINATE" -eq 1 ] && [ "$FOUND_COUNT" -gt 0 ]; then
        echo "--- TERMINATE: tehditler siliniyor ---"
        while IFS= read -r line; do
          f="''${line%: *}"
          if rm -f -- "$f" 2>/dev/null && [ ! -e "$f" ]; then
            REMOVED=$((REMOVED + 1))
            echo "SILINDI : $f"
            printf '%s  SILINDI %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$f" >> "$LOG"
          else
            REMOVE_FAILED=$((REMOVE_FAILED + 1))
            echo "SILINEMEDI: $f (salt-okunur dosya sistemi olabilir)" >&2
          fi
        done < "$TMP/found"
        echo ""
      fi

      if [ "$ERR_FILES" -gt 0 ]; then
        echo "--- ERISILEMEYEN DOSYALAR (ilk 5) ---"
        head -n 5 "$TMP/errlines"
        echo ""
      fi

      DATA="$(xargs -d '\n' -a "$TMP/all" stat -c %s 2>/dev/null | awk '{s+=$1} END{printf "%.2f MiB", s/1048576}' || true)"
      ENGINE="$(clamdscan --version 2>/dev/null | head -n 1 || true)"

      echo "----------- SCAN SUMMARY -----------"
      echo "Engine version: ''${ENGINE:-bilinmiyor}"
      echo "Scanned files: $TOTAL"
      echo "Infected files: $FOUND_COUNT"
      echo "Removed files: $REMOVED"
      echo "Remove failed: $REMOVE_FAILED"
      echo "Inaccessible files: $ERR_FILES"
      echo "Failed groups: $ERRORS"
      echo "Data scanned: ''${DATA:-?}"
      echo "Time: $ELAPSED sec ($((ELAPSED / 60)) m $((ELAPSED % 60)) s)"
      echo "Start Date: $START_DATE"
      echo "End Date:   $END_DATE"

      if [ "$FOUND_COUNT" -gt 0 ] && [ "$TERMINATE" -eq 0 ]; then
        echo ""
        echo "Silmek icin ayni komutu --TERMINATE ile tekrar calistir."
      fi

      if [ "$FOUND_COUNT" -gt 0 ]; then exit 1; fi
      if [ "$ERRORS" -gt 0 ]; then exit 2; fi
    '';
  };
in
{
  environment.systemPackages = [ cscan ];
}

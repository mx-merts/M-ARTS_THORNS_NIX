#!/usr/bin/env bash
set -e
COUNT_FILE=/etc/nixos/.rebuild-count
N=$(( $(cat "$COUNT_FILE" 2>/dev/null || echo 0) + 1 ))
echo "$N" > "$COUNT_FILE"
NIXOS_LABEL_VERSION="r${N}" nixos-rebuild switch --flake /etc/nixos#portable --impure "$@"

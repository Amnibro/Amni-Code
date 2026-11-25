#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
${HCN:-hcn} build dial.amni -o "${1:-$HOME/.local/bin/amni-dial}"

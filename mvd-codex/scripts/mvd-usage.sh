#!/usr/bin/env bash
# mvd-usage.sh - Show how mvd has been used (which harness, which commands, which repos).
#
# Usage: ./scripts/mvd-usage.sh [extra mvd usage args...]
#
# Examples:
#   ./scripts/mvd-usage.sh                       # last 30 days, grouped by harness
#   ./scripts/mvd-usage.sh --by command          # group by command name
#   ./scripts/mvd-usage.sh --by repo             # group by repository
#   ./scripts/mvd-usage.sh --since 7d --json
#   ./scripts/mvd-usage.sh --harness codex       # filter to one harness

set -euo pipefail

mvd usage "$@"

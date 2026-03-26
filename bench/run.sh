#!/usr/bin/env bash
set -euo pipefail

# Rblint Benchmark Script
# Usage: bench/run.sh <path-to-ruby-project> [label]

PROJECT_PATH="${1:?Usage: bench/run.sh <path-to-ruby-project> [label]}"
LABEL="${2:-$(basename "$PROJECT_PATH")}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RBLINT="$REPO_ROOT/target/release/rblint"

echo "Building release binary..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" 2>/dev/null

FILE_COUNT=$(find "$PROJECT_PATH" -name '*.rb' -type f | wc -l)
LINE_COUNT=$(find "$PROJECT_PATH" -name '*.rb' -type f -exec cat {} + 2>/dev/null | wc -l)

LEXER_RULES="R001,R002,R003,R010,R011,R012,R013,R020,R021,R022,R023,R024,R025,R026,R027,R028,R029,R030,R031,R033,R040,R041,R042,R043,R050,R051,R052,R053,R054"
AST_RULES="R060,R061,R062"

echo ""
echo "═══════════════════════════════════════════"
echo "Project: $LABEL"
printf "Files: %'d  Lines: %'d\n" "$FILE_COUNT" "$LINE_COUNT"
echo "─────────────────────────────────────────"

run_bench() {
    local label="$1"
    shift
    local args=("$@")

    if command -v hyperfine &>/dev/null; then
        hyperfine --warmup 2 --min-runs 5 \
            --style basic \
            "$RBLINT ${args[*]} --no-fail --no-cache $PROJECT_PATH" \
            2>&1 | grep 'Time (mean'
    else
        local total=0
        local runs=3
        for i in $(seq 1 $runs); do
            local start end elapsed
            start=$(date +%s%N)
            "$RBLINT" "${args[@]}" --no-fail --no-cache "$PROJECT_PATH" > /dev/null 2>&1
            end=$(date +%s%N)
            elapsed=$(( (end - start) / 1000000 ))
            total=$(( total + elapsed ))
        done
        local avg=$(( total / runs ))
        local secs avg_ms
        secs=$(( avg / 1000 ))
        avg_ms=$(( avg % 1000 ))
        printf "  %d.%03ds (avg of %d runs)\n" "$secs" "$avg_ms" "$runs"
    fi
}

echo ""
echo -n "Lexer rules only:  "
run_bench "lexer" --select "$LEXER_RULES"

echo -n "AST rules only:    "
run_bench "ast" --select "$AST_RULES"

echo -n "All rules:         "
run_bench "all"

DIAG_COUNT=$("$RBLINT" --no-fail --no-cache --format json "$PROJECT_PATH" 2>/dev/null | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "N/A")
echo ""
echo "Diagnostics: $DIAG_COUNT"
echo "═══════════════════════════════════════════"

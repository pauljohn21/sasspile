#!/usr/bin/env bash
# Run tests with tracing-ai compatible JSON output
# Usage:
#   ./test_with_tracing.sh                    # Run all tests
#   ./test_with_tracing.sh <test_name>         # Run specific test
#   ./test_with_tracing.sh --save <path>       # Save trace to specific path

set -euo pipefail

TRACE_LOG="/tmp/hrx_trace_$$.log"
TEST_OUTPUT="/tmp/hrx_test_output_$$.log"
SAVE_PATH=""

# Parse arguments
TEST_FILTER=""
for arg in "$@"; do
    case "$arg" in
        --save|-s)
            shift
            SAVE_PATH="${1:-}"
            ;;
        *)
            TEST_FILTER="$arg"
            ;;
    esac
done

echo "[tracing-ai] Running tests with JSON tracing enabled..."
echo "   Trace log: ${SAVE_PATH:-$TRACE_LOG}"

# Run tests with JSON tracing, capture stderr (trace JSON)
TRACING_AI_JSON=1 RUST_LOG=debug cargo test $TEST_FILTER 2>"$TRACE_LOG" | tee "$TEST_OUTPUT"
TEST_EXIT=${PIPESTATUS[0]}

# Copy to save path if specified
if [ -n "$SAVE_PATH" ]; then
    cp "$TRACE_LOG" "$SAVE_PATH"
    echo "   Trace saved to: $SAVE_PATH"
fi

echo ""
echo "[tracing-ai] Test Results:"
if [ $TEST_EXIT -eq 0 ]; then
    echo "   All tests passed"
else
    echo "   Tests failed (exit code: $TEST_EXIT)"
    echo ""
    echo "   To analyze failures with tracing-ai MCP:"
    echo "   1. Ask AI agent to 'analyze trace log at $TRACE_LOG'"
    echo "   2. Or pipe: cat $TRACE_LOG | tracing-ai --stdio"
fi

echo ""
echo "   Trace log: $TRACE_LOG"
echo "   Test output: $TEST_OUTPUT"

exit $TEST_EXIT

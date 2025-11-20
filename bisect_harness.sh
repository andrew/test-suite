#!/bin/bash
# Bisection script for testing harness changes
# Usage: ./bisect_harness.sh [start_commit] [end_commit] [test_name]
#
# Example: ./bisect_harness.sh cd0c1dc HEAD binary_file

set -e

START_COMMIT=${1:-"cd0c1dc"}
END_COMMIT=${2:-"HEAD"}
TEST_NAME=${3:-""}

echo "=========================================="
echo "Harness Bisection Tool"
echo "=========================================="
echo "Start commit: $START_COMMIT"
echo "End commit: $END_COMMIT"
echo "Test filter: ${TEST_NAME:-'(all tests)'}"
echo ""

# Get current branch/commit
CURRENT_REF=$(git rev-parse --abbrev-ref HEAD)
CURRENT_COMMIT=$(git rev-parse HEAD)

# Get list of commits affecting harness
echo "Finding commits affecting harness/harness.py..."
COMMITS=$(git log --oneline --reverse $START_COMMIT..$END_COMMIT -- harness/harness.py | awk '{print $1}')

if [ -z "$COMMITS" ]; then
    echo "No commits found in range"
    exit 1
fi

COMMIT_COUNT=$(echo "$COMMITS" | wc -l)
echo "Found $COMMIT_COUNT commit(s):"
echo "$COMMITS" | nl
echo ""

# Function to run test and extract metrics
run_test() {
    local commit=$1
    local commit_msg=$(git log -1 --format="%s" $commit)
    
    echo "----------------------------------------"
    echo "Testing: $commit"
    echo "Message: $commit_msg"
    echo "----------------------------------------"
    
    # Checkout commit
    git checkout -q $commit
    
    # Run harness
    local output_file="/tmp/harness_${commit}.json"
    local log_file="/tmp/harness_${commit}.log"
    
    if [ -n "$TEST_NAME" ]; then
        swhid-harness --payload "$TEST_NAME" --output-format canonical --dashboard-output "$output_file" > "$log_file" 2>&1 || true
    else
        swhid-harness --output-format canonical --dashboard-output "$output_file" > "$log_file" 2>&1 || true
    fi
    
    # Analyze results
    if [ -f "$output_file" ]; then
        python3 << PYEOF
import json
import sys

try:
    with open('$output_file', 'r') as f:
        data = json.load(f)
    
    tests = data.get('tests', [])
    total = len(tests)
    
    # Count tests where all implementations agree
    all_agree = 0
    disagreements = 0
    
    for test in tests:
        results = test.get('results', [])
        swhids = [r.get('swhid') for r in results if r.get('swhid')]
        statuses = [r.get('status') for r in results]
        
        # Check if all non-skipped implementations agree
        non_skipped = [r for r in results if r.get('status') != 'SKIPPED']
        if len(non_skipped) > 1:
            non_skipped_swhids = [r.get('swhid') for r in non_skipped if r.get('swhid')]
            if len(non_skipped_swhids) > 1 and len(set(non_skipped_swhids)) == 1:
                all_agree += 1
            else:
                disagreements += 1
    
    # Count by status
    pass_count = sum(1 for t in tests for r in t.get('results', []) if r.get('status') == 'PASS')
    fail_count = sum(1 for t in tests for r in t.get('results', []) if r.get('status') == 'FAIL')
    skip_count = sum(1 for t in tests for r in t.get('results', []) if r.get('status') == 'SKIPPED')
    
    print(f"Results:")
    print(f"  Total tests: {total}")
    print(f"  All agree: {all_agree}")
    print(f"  Disagreements: {disagreements}")
    print(f"  PASS: {pass_count}, FAIL: {fail_count}, SKIPPED: {skip_count}")
    
    # Show first disagreement if any
    if disagreements > 0:
        for test in tests:
            results = test.get('results', [])
            non_skipped = [r for r in results if r.get('status') != 'SKIPPED']
            if len(non_skipped) > 1:
                swhids = [r.get('swhid') for r in non_skipped if r.get('swhid')]
                if len(set(swhids)) > 1:
                    print(f"\n  First disagreement: {test.get('id')}")
                    for r in non_skipped:
                        print(f"    {r.get('implementation')}: {r.get('swhid', 'N/A')} ({r.get('status')})")
                    break
    
except Exception as e:
    print(f"Error analyzing results: {e}")
    sys.exit(1)
PYEOF
    else
        echo "  ERROR: Results file not generated"
        echo "  Last 10 lines of log:"
        tail -10 "$log_file" || echo "  (log file not found)"
    fi
    
    echo ""
}

# Test each commit
for commit in $COMMITS; do
    run_test $commit
done

# Return to original state
echo "=========================================="
echo "Returning to original state..."
git checkout -q $CURRENT_REF

echo ""
echo "Bisection complete!"
echo "Results saved in /tmp/harness_*.json"
echo "Logs saved in /tmp/harness_*.log"
echo ""
echo "To compare two commits:"
echo "  diff /tmp/harness_<commit1>.json /tmp/harness_<commit2>.json"


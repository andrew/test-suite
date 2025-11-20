# Harness Bisection Process

## Overview
This document describes the process for bisecting issues in the harness test matrix.

## Quick Start

### 1. Run Bisection Script
```bash
./bisect_harness.sh [start_commit] [end_commit] [test_name]
```

Example:
```bash
./bisect_harness.sh cd0c1dc HEAD binary_file
```

### 2. Manual Bisection
```bash
# Test a specific commit
git checkout <commit>
swhid-harness --payload <test_name> --output-format canonical --dashboard-output /tmp/results.json

# Analyze results
python3 -c "
import json
with open('/tmp/results.json') as f:
    data = json.load(f)
    tests = data.get('tests', [])
    for t in tests:
        results = t.get('results', [])
        swhids = [r.get('swhid') for r in results if r.get('swhid')]
        print(f\"{t.get('id')}: {len(set(swhids))} unique SWHIDs\")
"
```

## Analyzing Results

### Check for Regressions
1. Compare `all_agree` count between commits
2. Check for new disagreements
3. Verify expected SWHIDs still match

### Common Issues to Check
- Path resolution changes affecting payload access
- Logic changes in `_compare_results`
- Summary calculation bugs
- Status determination in `get_canonical_results`

## Recent Changes to Review

### Commits affecting harness/harness.py (last 6 hours)
- `d81ed66`: Fix synthetic_repo creation: check for valid Git repo
- `6113782`: Fix synthetic_repo path resolution issue  
- `cd0c1dc`: Fix Windows CI issues: Unicode encoding and Rust binary path
- `a21ad22`: fix: ensure deterministic synthetic_repo creation

### Key Changes
1. **Path Resolution** (6113782): Changed from relative to absolute paths
   - May affect how payloads are accessed
   - Check if paths are resolved correctly

2. **Synthetic Repo Creation** (d81ed66, 6113782): Enhanced Git repo detection
   - Changed from `os.path.exists()` to checking for valid Git repo
   - May affect when synthetic repos are created

3. **Summary Logic** (existing): The `_print_summary` function calculates "all agree" 
   - Line 1114: `len(swhids) <= 1` should be `len(swhids) == 1`
   - Line 1133: Checks expected SWHID match per result

## Debugging Tips

1. **Check ComparisonResult.all_match**: This is set by `_compare_results()`
2. **Check Summary Calculation**: `_print_summary()` recalculates from canonical results
3. **Verify Path Resolution**: Ensure absolute paths are used consistently
4. **Check Expected SWHIDs**: Verify they match actual computed values


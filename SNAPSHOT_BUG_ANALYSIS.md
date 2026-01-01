# Snapshot SWHID Bug Analysis

## Problem

The `synthetic_repo` test fails with:
- **Expected**: `swh:1:snp:d66db619f7d82b4bc6524810d045d731ae099ef7`
- **Got**: `swh:1:snp:3ab1765bb1fa4dc4e07e245b39e3853ebbfd6ada`

Both `rust` and `ruby` implementations produce the same wrong SWHID, suggesting a bug in `swhid-rs`.

## Root Cause

### Location: `swhid-rs/src/git.rs`

**Function**: `reference_to_branch_target()` (lines 581-649)

**The Bug**: For Direct references, the function uses Git object OIDs directly instead of computing SWHID digests:

```rust
Some(git2::ObjectType::Commit) => BranchTarget::Revision(Some(target_id)),  // ❌ Uses Git OID
Some(git2::ObjectType::Tag) => BranchTarget::Release(Some(target_id)),      // ❌ Uses Git OID
```

**What Should Happen**: The snapshot manifest should contain **SWHID digests**, not Git OIDs:
- For commits: Compute revision SWHID, use its digest
- For tags: Compute release SWHID, use its digest

### Evidence

From the synthetic repo:
- **Main branch commit**: `6436a7e7facae7f0d9c115808d5e85d011c631d9` (Git OID)
- **Feature branch commit**: `9ea876cdf36b151aa030724ea6e09fcd035f1550` (Git OID)
- **Tag v1.0 object**: `6cec7691b701970c4de255441451a600169425ae` (Git OID)

The snapshot is currently using these Git OIDs directly in the manifest, but it should use:
- Revision SWHID digests (computed from commits)
- Release SWHID digest (computed from tag object)

### Snapshot Manifest Format

According to `snapshot.rs`, the manifest format is:
```
<type> <name>\0<len>:<digest>
```

Where:
- `<type>` is "revision", "release", etc.
- `<name>` is the branch/tag name
- `<digest>` should be the **SWHID digest** (20 bytes for SHA1), not the Git OID

### The Fix

The `reference_to_branch_target()` function needs to:
1. For commits: Compute revision SWHID using `revision_swhid()`, extract digest
2. For tags: Compute release SWHID using `release_swhid()`, extract digest
3. Use these SWHID digests in `BranchTarget`, not the Git OIDs

### Impact

This affects all snapshot computations in `swhid-rs`. The bug is in the upstream `swhid-rs` codebase, not in our test harness implementation.

## Next Steps

1. **Report bug to swhid-rs**: This is a bug in the upstream `swhid-rs` repository
2. **Workaround**: None available in our test harness - we're calling `swhid git snapshot` correctly
3. **Verification**: Once fixed in `swhid-rs`, the test should pass

## Repository Structure (for reference)

The synthetic repo has:
- `refs/heads/main` → commit `6436a7e7facae7f0d9c115808d5e85d011c631d9`
- `refs/heads/feature` → commit `9ea876cdf36b151aa030724ea6e09fcd035f1550`
- `refs/tags/v1.0` → tag object `6cec7691b701970c4de255441451a600169425ae` (points to main commit)

The snapshot should include these three references, sorted by name:
1. `feature` → revision SWHID digest
2. `main` → revision SWHID digest  
3. `v1.0` → release SWHID digest


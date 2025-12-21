# Comprehensive Analysis: Windows Permission Issues

## Executive Summary

**Problem**: All implementations (git, git-cmd, rust, ruby) produce the same SWHID on Windows for permission tests, but it differs from the expected Unix-based SWHID.

**Root Cause**: Windows doesn't preserve Unix executable bits when copying files, and Git on Windows doesn't reliably detect executable permissions.

**Solution**: Preserve the intended permissions from source files and apply them in a Windows-native way before Git processes them.

## Current Situation

### Test Results
- **Expected SWHID** (Unix): `swh:1:dir:32798ac33695bd283d6e650c61a40bc2dbda3a2e` (comprehensive_permissions)
- **Actual SWHID** (Windows): `swh:1:dir:bbb968463c32959031f5962fef0cd51530dcf194`
- **All implementations agree** on Windows (consistent but wrong)

### Test Payload Structure

The `comprehensive_permissions` test contains:
- `executable.txt` - Should be executable (mode 100755)
- `readonly.txt` - Regular file (mode 100644)
- `regular.txt` - Regular file (mode 100644)
- `writeonly.txt` - Regular file (mode 100644)

## Root Cause Analysis

### 1. Windows File System Limitations

**Windows uses ACLs (Access Control Lists) instead of Unix permissions:**
- No native executable bit
- Permissions are user/group-based, not mode-based
- File extensions determine executability (.exe, .bat, etc.)

**When files are copied:**
- Unix executable bits are **not preserved** on Windows filesystems
- `shutil.copy2()` preserves metadata, but Windows doesn't have executable bits to preserve
- Files copied from Unix to Windows lose their executable status

### 2. Git Behavior on Windows

**Git's `core.filemode` setting:**
- `core.filemode=true`: Git tracks permission changes (default on Unix)
- `core.filemode=false`: Git ignores permission changes (default on Windows)

**Why Git ignores permissions on Windows by default:**
- Windows filesystem doesn't reliably support executable bits
- Git would show false "permission changed" diffs
- Most Windows users don't need permission tracking

**When `core.filemode=true` is set on Windows:**
- Git still can't reliably detect executable bits
- May assign default permissions (644 for all files)
- Or may use file extension heuristics

### 3. Implementation Behavior

**Current implementations:**
1. **git-cmd**: Uses `git add` which relies on Git's permission detection
2. **git (dulwich)**: Uses `os.stat().st_mode & stat.S_IEXEC` + file extension fallback
3. **rust**: Uses Rust tool which may have similar issues
4. **ruby**: Uses Ruby gem which may have similar issues

**The problem:**
- On Windows, `os.stat().st_mode & stat.S_IEXEC` is unreliable
- File extension detection only works for `.exe`, `.bat`, etc. (not `.txt`)
- The test payload has `executable.txt` which should be executable but has `.txt` extension

## Solution: Preserve Intended Permissions

### Approach: Read Source Permissions Before Copy

The "natural" way on Windows is to:
1. **Read the intended permissions from the source files** (before copying)
2. **Store this information** (the source files have the correct permissions)
3. **Apply it when creating Git trees** (use the stored information, not filesystem detection)

### Implementation Strategy

#### Option 1: Read Permissions from Source (Recommended)

**For git-cmd:**
- Before copying, read permissions from source files
- Store which files should be executable
- After `git add`, use `git update-index --chmod=+x` to set executable bits

**For git (dulwich):**
- Before copying, read permissions from source files
- Use stored permission information when creating tree entries
- Don't rely on `os.stat()` on Windows

**For rust/ruby:**
- Similar approach: read source permissions, apply when computing SWHID

#### Option 2: Use Git Index Manipulation

**For git-cmd:**
```bash
# After git add, explicitly set executable bits
git update-index --chmod=+x executable.txt
```

**Pros:**
- Uses Git's native mechanism
- Works with `core.filemode=true`

**Cons:**
- Requires knowing which files should be executable
- Adds extra Git commands

#### Option 3: Preserve Permissions During Copy

**Use a custom copy function that:**
1. Reads source file permissions
2. Copies file content
3. On Windows, stores permission info separately
4. Applies when needed

**Pros:**
- Transparent to implementations
- Works for all implementations

**Cons:**
- More complex
- Requires coordination between copy and Git operations

## Recommended Solution

### Step 1: Detect Source Permissions

Create a utility function that reads permissions from source files:

```python
def get_source_file_permissions(source_dir):
    """Read intended permissions from source files."""
    permissions = {}
    for root, dirs, files in os.walk(source_dir):
        for file in files:
            file_path = os.path.join(root, file)
            rel_path = os.path.relpath(file_path, source_dir)
            stat_info = os.stat(file_path)
            is_executable = bool(stat_info.st_mode & stat.S_IEXEC)
            permissions[rel_path] = is_executable
    return permissions
```

### Step 2: Apply Permissions in git-cmd

```python
# After copying files but before git add
source_permissions = get_source_file_permissions(dir_path)

# Copy files
shutil.copytree(dir_path, target_path, symlinks=True)

# After git add, apply executable bits
for rel_path, is_executable in source_permissions.items():
    if is_executable:
        file_path = os.path.join(repo_path, rel_path)
        if os.path.exists(file_path):
            subprocess.run(
                ["git", "update-index", "--chmod=+x", rel_path],
                cwd=repo_path, check=True, capture_output=True
            )
```

### Step 3: Apply Permissions in git (dulwich)

```python
# Read source permissions before copying
source_permissions = get_source_file_permissions(dir_path)

# When creating tree entries, use source permissions
for item in os.listdir(dir_path):
    item_path = os.path.join(dir_path, item)
    rel_path = os.path.relpath(item_path, dir_path)
    
    # Use source permission if available, otherwise detect
    is_executable = source_permissions.get(rel_path, False)
    if not is_executable and platform.system() != 'Windows':
        # On Unix, also check filesystem
        is_executable = bool(os.stat(item_path).st_mode & stat.S_IEXEC)
    
    mode = 0o100755 if is_executable else 0o100644
```

### Step 4: Handle Windows-Specific Cases

**For files with executable intent but non-executable extensions:**
- Trust the source file's permission
- Don't rely on file extension heuristics for test payloads
- The source files are the source of truth

## Implementation Details

### Key Insight

The test payload files are created on a Unix-like system with explicit permissions:
- `executable.txt` has mode `100755` (executable)
- Other files have mode `100644` (regular)

**These permissions are the intended state**, regardless of platform.

### Windows-Native Approach

On Windows, we should:
1. **Respect the source file's intended permissions** (read from source)
2. **Apply them using Git's mechanisms** (`git update-index --chmod`)
3. **Not rely on Windows filesystem detection** (unreliable)

This is "natural" on Windows because:
- We're using Git's permission model (not Windows ACLs)
- We're preserving the intended state from source files
- We're not fighting Windows filesystem limitations

## Testing Strategy

### Verification Steps

1. **On Linux/Unix**: Verify current behavior (should work)
2. **On Windows**: 
   - Read source permissions
   - Apply via Git mechanisms
   - Verify SWHID matches expected

### Test Cases

1. **comprehensive_permissions**: All file types
2. **permissions_dir**: Directory with permission variations
3. **mixed_types**: Files + executables + symlinks

## Expected Outcomes

After implementation:
- ✅ Windows implementations produce same SWHID as Unix
- ✅ All implementations agree across platforms
- ✅ Permissions are preserved from source files
- ✅ Works "naturally" on Windows (using Git's mechanisms)

## Alternative: Platform-Specific Expected Values

**If preserving permissions is too complex**, consider:
- Platform-specific expected SWHIDs in `config.yaml`
- Document that Windows produces different (but consistent) SWHIDs
- Accept platform differences as expected behavior

**However**, this goes against the goal of "obtaining the same results for the same payloads".

## Conclusion

The solution is to **read permissions from source files** and **apply them using Git's mechanisms** before computing SWHIDs. This is the "natural" way on Windows because:

1. We respect the source file's intended state
2. We use Git's permission model (not Windows ACLs)
3. We get consistent results across platforms
4. We work with Windows filesystem limitations, not against them

The key is treating the **source files as the source of truth** for permissions, not the filesystem after copying.


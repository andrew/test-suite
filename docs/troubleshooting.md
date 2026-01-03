# Troubleshooting Guide

## Common Issues and Solutions

### Implementation Not Found

**Symptoms**:
- Error: "Implementation not found" or "Implementation not available"
- Implementation missing from `--list-impls` output

**Solutions**:

1. **Check Implementation Installation**:
   ```bash
   # Python
   python3 -c "import swh.model"
   
   # Ruby
   gem list swhid
   
   # Rust
   cargo --version
   ```

2. **Check PATH**:
   ```bash
   # Verify binary is in PATH
   which swhid  # Ruby
   which cargo  # Rust
   ```

3. **Check Environment Variables**:
   ```bash
   # Rust implementation
   echo $SWHID_RS_PATH
   
   # Ruby implementation
   echo $GEM_HOME
   ```

4. **Verify Implementation Directory**:
   ```bash
   ls implementations/python/implementation.py
   ```

### Test Failures

**Symptoms**:
- Tests failing with unexpected errors
- Implementations producing different SWHIDs

**Solutions**:

1. **Check Test Payload**:
   ```bash
   # Verify payload exists
   ls payloads/content/basic/hello_world
   ```

2. **Check Implementation Output**:
   ```bash
   # Run implementation directly
   python3 -m swh.model.cli --type content payloads/content/basic/hello_world
   ```

3. **Check Logs**:
   ```bash
   # Run with verbose logging
   python3 main.py --category content --payload hello_world -v
   ```

4. **Compare Implementations**:
   ```bash
   # Run single implementation
   python3 main.py --category content --implementation python
   ```

### Permission Issues (Windows)

**Symptoms**:
- Directory tests failing on Windows
- Executable permissions not preserved

**Solutions**:

1. **Check Git Configuration**:
   ```bash
   git config core.filemode
   # Should be 'true'
   ```

2. **Verify Git Index**:
   ```bash
   git ls-files --stage path/to/file
   # Check mode (should be 100755 for executables)
   ```

3. **Check Implementation Logs**:
   - Ruby/Rust implementations create temporary Git repos
   - Check logs for permission-related warnings

### Timeout Errors

**Symptoms**:
- Tests timing out
- "Operation timed out" errors

**Solutions**:

1. **Increase Timeout**:
   ```yaml
   # In config.yaml
   settings:
     timeout: 60  # Increase from default 30
   ```

2. **Check System Resources**:
   ```bash
   # Check CPU/memory usage
   top  # or htop
   ```

3. **Reduce Parallelism**:
   ```yaml
   # In config.yaml
   settings:
     parallel_tests: 2  # Reduce from default 4
   ```

4. **Run Single Test**:
   ```bash
   # Test specific payload
   python3 main.py --category content --payload hello_world
   ```

### Configuration Errors

**Symptoms**:
- "Invalid configuration file" errors
- Configuration validation failures

**Solutions**:

1. **Validate YAML Syntax**:
   ```bash
   # Check YAML syntax
   python3 -c "import yaml; yaml.safe_load(open('config.yaml'))"
   ```

2. **Check Required Fields**:
   - Ensure all required fields are present
   - Check field types match expected types

3. **Verify Paths**:
   ```bash
   # Check payload paths exist
   python3 -c "from harness.config import HarnessConfig; HarnessConfig.load_from_file('config.yaml')"
   ```

### Memory Issues

**Symptoms**:
- Out of memory errors
- Tests killed by system

**Solutions**:

1. **Reduce Parallelism**:
   ```yaml
   settings:
     parallel_tests: 1
   ```

2. **Clean Up Temporary Files**:
   ```bash
   # Remove old results
   rm -rf results/*
   ```

3. **Check Resource Limits**:
   ```bash
   # Increase limits if possible
   ulimit -v  # Check virtual memory limit
   ```

### Cross-Platform Issues

**Symptoms**:
- Tests pass on one platform but fail on another
- Line ending issues
- Path separator issues

**Solutions**:

1. **Check Line Endings**:
   ```bash
   # Verify Git configuration
   git config core.autocrlf
   # Should be 'false' for SWHID testing
   ```

2. **Check Path Separators**:
   - Use forward slashes in Git operations
   - Normalize paths before use

3. **Check File Permissions**:
   - Windows: Use Git index for permissions
   - Unix: Use filesystem permissions

### Python Implementation Issues

**Symptoms**:
- "swh.model not available" errors
- Python implementation not working

**Solutions**:

1. **Install swh.model**:
   ```bash
   pip install swh.model
   ```

2. **Check Python Version**:
   ```bash
   python3 --version
   # Should be 3.8+
   ```

3. **Verify Installation**:
   ```bash
   python3 -m swh.model.cli --help
   ```

4. **Check GPG Signature Support**:
   - Signed tags are skipped (limitation)
   - Signed commits are supported

### Ruby Implementation Issues

**Symptoms**:
- "swhid command not found"
- Ruby implementation not available

**Solutions**:

1. **Install Ruby Gem**:
   ```bash
   gem install swhid
   ```

2. **Check GEM_HOME**:
   ```bash
   echo $GEM_HOME
   gem env
   ```

3. **Verify Binary Path**:
   ```bash
   # Check gem-specific paths
   ls ~/.gem/ruby/*/bin/swhid
   ```

4. **Check Windows Permissions**:
   - Ruby implementation uses Git index for permissions
   - Ensure Git is properly configured

### Rust Implementation Issues

**Symptoms**:
- "cargo not found" errors
- Rust binary not available

**Solutions**:

1. **Build Rust Implementation**:
   ```bash
   cd implementations/rust
   cargo build --release
   ```

2. **Set SWHID_RS_PATH**:
   ```bash
   export SWHID_RS_PATH=/path/to/target/release/swhid
   ```

3. **Check Binary Path**:
   ```bash
   # Verify binary exists and is executable
   ls -l $SWHID_RS_PATH
   ```

4. **Check Command Format**:
   - Rust implementation supports both positional and flag-based arguments
   - Auto-detection handles both formats

### Git Implementation Issues

**Symptoms**:
- Git commands failing
- Repository creation errors

**Solutions**:

1. **Check Git Installation**:
   ```bash
   git --version
   ```

2. **Check Git Configuration**:
   ```bash
   git config --global user.name
   git config --global user.email
   ```

3. **Verify Repository Creation**:
   ```bash
   # Test repository creation
   python3 -c "from harness.git_manager import GitManager; gm = GitManager(); gm.create_minimal_git_repo('/tmp/test_repo')"
   ```

### Output Generation Issues

**Symptoms**:
- JSON output malformed
- Summary not printing correctly

**Solutions**:

1. **Check JSON Validity**:
   ```bash
   python3 -m json.tool results/results_*.json
   ```

2. **Verify Pydantic Models**:
   ```bash
   python3 -c "from harness.models import HarnessResults; import json; json.load(open('results/results_*.json'))"
   ```

3. **Check Output Directory**:
   ```bash
   # Ensure results directory exists and is writable
   mkdir -p results
   chmod 755 results
   ```

## Debugging Tips

### Enable Verbose Logging

```bash
# Set logging level
export PYTHONPATH=.
python3 main.py --category content -v
```

### Run Single Test

```bash
# Test specific payload
python3 main.py --category content --payload hello_world
```

### Check Implementation Availability

```bash
# List all implementations
python3 main.py --list-impls
```

### Inspect Test Results

```bash
# View JSON results
cat results/results_*.json | python3 -m json.tool | less
```

### Test Implementation Directly

```bash
# Python
python3 -m swh.model.cli --type content payloads/content/basic/hello_world

# Ruby
swhid content payloads/content/basic/hello_world

# Rust
swhid content payloads/content/basic/hello_world
```

## Getting Help

### Check Documentation

- [Developer Guide](../DEVELOPER_GUIDE.md): Comprehensive usage guide
- [Implementation Details](../IMPLEMENTATIONS.md): Implementation-specific information
- [Platform Limitations](../PLATFORM_LIMITATIONS.md): Known limitations

### Report Issues

1. **Collect Information**:
   - Error messages
   - Log output
   - System information (OS, Python version, etc.)
   - Configuration file (sanitized)

2. **Reproduce Issue**:
   - Minimal test case
   - Steps to reproduce
   - Expected vs actual behavior

3. **Check Existing Issues**:
   - Search for similar issues
   - Check if already fixed

## Common Error Messages

### "Object type 'X' not supported by implementation"

**Cause**: Implementation doesn't support the requested object type.

**Solution**: Check implementation capabilities:
```bash
python3 main.py --list-impls
```

### "Payload file not found"

**Cause**: Test payload doesn't exist or path is incorrect.

**Solution**: Verify payload path in config.yaml and filesystem.

### "Implementation timed out"

**Cause**: Test execution exceeded timeout limit.

**Solution**: Increase timeout in config.yaml or check system resources.

### "Invalid SWHID format"

**Cause**: Implementation returned malformed SWHID.

**Solution**: Check implementation output directly, verify implementation is working correctly.

### "No implementations available"

**Cause**: No implementations found or all unavailable.

**Solution**: Check implementation installation and availability.


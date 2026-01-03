# SWHID Testing Harness Architecture

## Overview

The SWHID Testing Harness is a comprehensive testing framework for comparing different Software Heritage Identifier (SWHID) implementations. It uses a plugin-based architecture that enables easy extension with new implementations.

## System Architecture

### High-Level Data Flow

```
┌─────────────┐
│ config.yaml │  Configuration file defining test payloads
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│  SwhidHarness    │  Main orchestrator
│  - Load config   │
│  - Discover impls│
│  - Run tests     │
└──────┬───────────┘
       │
       ├─────────────────────────────────┐
       │                                 │
       ▼                                 ▼
┌──────────────────┐          ┌──────────────────┐
│ Implementation   │          │ Implementation   │
│ Discovery        │          │ Discovery        │
│ - Find plugins   │          │ - Find plugins   │
│ - Load modules   │          │ - Load modules   │
└──────┬───────────┘          └──────┬───────────┘
       │                              │
       ▼                              ▼
┌──────────────────┐          ┌──────────────────┐
│ Python Impl      │          │ Rust Impl        │
│ - compute_swhid()│          │ - compute_swhid()│
└──────┬───────────┘          └──────┬───────────┘
       │                              │
       └──────────────┬───────────────┘
                      │
                      ▼
            ┌──────────────────┐
            │ SwhidTestResult  │  Individual test results
            └────────┬─────────┘
                     │
                     ▼
            ┌──────────────────┐
            │ ResultComparator │  Compare results across impls
            └────────┬─────────┘
                     │
                     ▼
            ┌──────────────────┐
            │ ComparisonResult  │  Aggregated comparison
            └────────┬─────────┘
                     │
                     ▼
            ┌──────────────────┐
            │ OutputGenerator  │  Format results
            └────────┬─────────┘
                     │
                     ▼
            ┌──────────────────┐
            │ HarnessResults   │  Canonical JSON output
            │ (Pydantic)       │
            └──────────────────┘
```

## Component Architecture

### Core Components

#### 1. SwhidHarness (`harness/harness.py`)

**Purpose**: Main orchestrator that coordinates all testing activities.

**Responsibilities**:
- Configuration loading and validation
- Implementation discovery and management
- Test execution orchestration
- Result aggregation and output generation

**Key Methods**:
- `run_tests()`: Execute tests for specified implementations and categories
- `generate_expected_results()`: Generate expected SWHID values
- `get_canonical_results()`: Generate canonical JSON format results

#### 2. TestRunner (`harness/runner.py`)

**Purpose**: Handles test execution logic.

**Responsibilities**:
- Execute individual tests
- Manage parallel test execution
- Handle test timeouts and errors
- Coordinate with implementations

**Key Methods**:
- `run_single_test()`: Execute a single test for one implementation
- `run_tests()`: Execute tests for all implementations

#### 3. ResultComparator (`harness/comparator.py`)

**Purpose**: Compare test results across implementations.

**Responsibilities**:
- Detect consensus among implementations
- Identify disagreements
- Handle unsupported object types
- Validate against expected SWHIDs

**Key Methods**:
- `compare_results()`: Compare results across implementations
- `is_unsupported_result()`: Detect skipped/unsupported tests

#### 4. OutputGenerator (`harness/output.py`)

**Purpose**: Generate formatted output from test results.

**Responsibilities**:
- Generate canonical JSON format
- Create summary reports
- Format error information
- Calculate aggregate statistics

**Key Methods**:
- `get_canonical_results()`: Generate canonical format
- `print_summary()`: Print human-readable summary

#### 5. ResourceManager (`harness/resource_manager.py`)

**Purpose**: Manage temporary resources.

**Responsibilities**:
- Temporary directory management
- Tarball extraction
- Resource cleanup (cross-platform)

**Key Methods**:
- `extract_tarball_if_needed()`: Extract tarball payloads
- `cleanup_temp_dirs()`: Clean up temporary directories

#### 6. GitManager (`harness/git_manager.py`)

**Purpose**: Handle Git repository operations.

**Responsibilities**:
- Create synthetic Git repositories
- Resolve commit references
- Discover branches and tags

**Key Methods**:
- `create_minimal_git_repo()`: Create test Git repository
- `discover_git_tests()`: Discover branches/tags for testing

### Plugin System

#### Implementation Discovery (`harness/plugins/discovery.py`)

**Purpose**: Discover and load SWHID implementations.

**How it works**:
1. Scans `implementations/` directory
2. Loads Python modules dynamically
3. Instantiates implementation classes
4. Validates implementation availability

#### Base Interface (`harness/plugins/base.py`)

**Purpose**: Define the interface all implementations must follow.

**Key Classes**:
- `SwhidImplementation`: Abstract base class
- `ImplementationInfo`: Metadata about an implementation
- `ImplementationCapabilities`: Supported features
- `SwhidTestResult`: Test result structure

**Required Methods**:
- `get_info()`: Return implementation metadata
- `is_available()`: Check if implementation is available
- `get_capabilities()`: Return supported features
- `compute_swhid()`: Compute SWHID for a payload

### Utility Modules

#### Constants (`harness/utils/constants.py`)

**Purpose**: Centralized constants and enums.

**Contents**:
- SWHID type codes (cnt, dir, rev, rel, snp)
- Object types
- Test status values
- Timeout constants

#### Subprocess Utilities (`harness/utils/subprocess_utils.py`)

**Purpose**: Shared subprocess execution utilities.

**Functions**:
- `prepare_subprocess_environment()`: Set up environment
- `set_resource_limits()`: Set resource limits
- `run_with_timeout()`: Execute with timeout

#### Git Utilities (`harness/utils/git_utils.py`)

**Purpose**: Shared Git operations.

**Functions**:
- `resolve_commit_reference()`: Resolve commit refs to SHAs
- `discover_branches()`: Discover repository branches
- `discover_annotated_tags()`: Discover annotated tags
- `is_git_repository()`: Check if path is a Git repo

#### Permission Utilities (`harness/utils/permissions.py`)

**Purpose**: Cross-platform permission handling.

**Functions**:
- `get_source_permissions()`: Read permissions from Git index or filesystem
- `create_git_repo_with_permissions()`: Create Git repo with permissions

### Configuration System

#### Config Models (`harness/config.py`)

**Purpose**: Type-safe configuration validation using Pydantic.

**Key Models**:
- `HarnessConfig`: Main configuration structure
- `PayloadConfig`: Test payload definition
- `OutputConfig`: Output settings
- `SettingsConfig`: General settings

**Features**:
- Automatic validation
- Type safety
- Default values
- Path resolution

### Data Models

#### Result Models (`harness/models.py`)

**Purpose**: Pydantic models for canonical result format.

**Key Models**:
- `HarnessResults`: Top-level result structure
- `TestCase`: Individual test case result
- `Result`: Implementation-specific result
- `ErrorInfo`: Structured error information
- `Metrics`: Performance metrics

## Implementation Architecture

### Implementation Structure

Each implementation follows a consistent structure:

```
implementations/
├── python/
│   ├── __init__.py
│   └── implementation.py  # Python implementation plugin
├── rust/
│   ├── __init__.py
│   └── implementation.py  # Rust implementation plugin
└── ...
```

### Implementation Interface

All implementations must:

1. **Inherit from `SwhidImplementation`**
2. **Implement required methods**:
   - `get_info()`: Return metadata
   - `is_available()`: Check availability
   - `get_capabilities()`: Return capabilities
   - `compute_swhid()`: Compute SWHID

3. **Handle errors appropriately**:
   - Use custom exceptions from `harness.exceptions`
   - Provide clear error messages
   - Support timeout handling

### Example Implementation Flow

```
TestRunner.run_single_test()
    ↓
Implementation.compute_swhid(payload_path, obj_type, ...)
    ↓
[Implementation-specific logic]
    - May use subprocess (git, cargo, ruby)
    - May use Python libraries (swh.model, pygit2)
    - May create temporary resources
    ↓
Return SWHID string or raise exception
    ↓
SwhidTestResult (success=True/False, swhid, error, ...)
```

## Error Handling

### Exception Hierarchy

```
SwhidHarnessError (base)
├── ConfigurationError
├── ImplementationError
│   └── ProtocolError
├── TestExecutionError
│   ├── TimeoutError
│   └── ResourceLimitError
└── ResultError
```

### Error Classification

Errors are automatically classified into:
- **Error Codes**: TIMEOUT, RESOURCE_LIMIT, IO_ERROR, etc.
- **Subtypes**: file_not_found, permission_denied, etc.
- **Context**: Additional debugging information

## Cross-Platform Considerations

### Windows-Specific Handling

1. **File Permissions**:
   - Use Git index to preserve executable permissions
   - Create temporary Git repositories with permissions
   - Shared utilities in `harness/utils/permissions.py`

2. **Path Handling**:
   - Normalize path separators
   - Handle UNC paths
   - Use forward slashes for Git operations

3. **Line Endings**:
   - Configure Git with `core.autocrlf=false`
   - Preserve original line endings

### Unix/MacOS Handling

1. **File Permissions**:
   - Read from filesystem directly
   - Preserve permissions when copying

2. **Resource Limits**:
   - Use `setrlimit()` for memory/CPU limits
   - Signal-based timeouts

## Testing Architecture

### Test Execution Flow

```
1. Load configuration (config.yaml)
2. Discover implementations
3. For each test payload:
   a. Extract if tarball
   b. For each implementation:
      - Run compute_swhid()
      - Collect result
   c. Compare results
   d. Check against expected SWHID
4. Generate output (JSON + summary)
```

### Parallel Execution

- Uses `ThreadPoolExecutor` for parallel test execution
- Configurable via `settings.parallel_tests`
- Each test runs independently
- Results collected asynchronously

## Extension Points

### Adding a New Implementation

1. Create directory: `implementations/your_impl/`
2. Create `implementation.py` with `Implementation` class
3. Inherit from `SwhidImplementation`
4. Implement required methods
5. Register in discovery (automatic)

### Adding New Test Categories

1. Add category to `config.yaml`
2. Define payloads in category
3. Run with `--category your_category`

### Custom Output Formats

1. Extend `OutputGenerator`
2. Add new format method
3. Update CLI to support format

## Performance Considerations

### Caching

- Implementation discovery cached
- Binary paths cached
- Git repository creation optimized

### Resource Management

- Temporary directories cleaned up automatically
- Resource limits enforced
- Timeout handling prevents hangs

### Parallel Execution

- Configurable parallelism
- Thread-safe result collection
- Independent test execution

## Security Considerations

### Subprocess Execution

- Clean environment preparation
- Timeout enforcement
- Resource limit enforcement
- Input validation

### File System Access

- Path validation
- Temporary directory isolation
- Cleanup on errors

## Future Enhancements

### Potential Improvements

1. **Distributed Testing**: Run tests across multiple machines
2. **Result Caching**: Cache results for unchanged payloads
3. **Incremental Testing**: Only test changed implementations
4. **Performance Profiling**: Detailed performance metrics
5. **Visualization**: Interactive result visualization
6. **CI/CD Integration**: Better GitHub Actions integration

## Related Documentation

- [Developer Guide](../DEVELOPER_GUIDE.md): How to use and extend the harness
- [Implementation Details](../IMPLEMENTATIONS.md): Details about each implementation
- [Platform Limitations](../PLATFORM_LIMITATIONS.md): Known limitations and workarounds


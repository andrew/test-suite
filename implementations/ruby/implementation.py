"""
Ruby SWHID Implementation Plugin

This module provides an interface to the Ruby SWHID implementation
for the testing harness.
"""

import subprocess
import os
import platform
import logging
from typing import Optional

from harness.plugins.base import SwhidImplementation, ImplementationInfo, ImplementationCapabilities

logger = logging.getLogger(__name__)

class Implementation(SwhidImplementation):
    """Ruby SWHID implementation plugin."""
    
    def __init__(self):
        """Initialize Ruby implementation and find swhid command path."""
        super().__init__()
        self._swhid_path = None
        self._temp_dirs: list = []  # Track temp directories for cleanup
        self._find_swhid_path()
    
    def _find_swhid_path(self) -> Optional[str]:
        """Find the swhid command path and cache it.
        
        Prefers Ruby gem's swhid over Rust binary by checking gem-specific
        paths first, then falling back to PATH search.
        """
        if self._swhid_path:
            logger.debug(f"Ruby: Using cached swhid path: {self._swhid_path}")
            return self._swhid_path
        
        import shutil
        import os
        import glob
        
        logger.debug("Ruby: Starting swhid binary detection")
        is_windows = platform.system() == "Windows"
        logger.debug(f"Ruby: Platform is Windows: {is_windows}")
        
        # CRITICAL: Check gem-specific paths FIRST to prefer Ruby gem over Rust binary
        # The Rust binary may be in PATH and come first, but we need the Ruby gem
        home = os.path.expanduser("~")
        gem_paths = [
            os.path.join(home, ".gem", "ruby", "*", "bin", "swhid"),
            os.path.join(home, ".local", "share", "gem", "ruby", "*", "bin", "swhid"),
        ]
        logger.debug(f"Ruby: Checking standard gem paths: {gem_paths}")
        
        # Also try GEM_HOME if set (used by ruby/setup-ruby)
        gem_home = os.environ.get("GEM_HOME")
        if gem_home:
            logger.debug(f"Ruby: GEM_HOME is set: {gem_home}")
            # Normalize path separators for Windows compatibility
            gem_home_normalized = os.path.normpath(gem_home)
            gem_home_bin = os.path.join(gem_home_normalized, "bin", "swhid")
            gem_paths.append(gem_home_bin)
            logger.debug(f"Ruby: Added GEM_HOME path: {gem_home_bin}")
            # Also check if the bin directory exists
            gem_home_bin_dir = os.path.join(gem_home_normalized, "bin")
            if os.path.isdir(gem_home_bin_dir):
                logger.debug(f"Ruby: GEM_HOME/bin directory exists: {gem_home_bin_dir}")
                # List files in bin directory for debugging
                try:
                    bin_files = os.listdir(gem_home_bin_dir)
                    logger.debug(f"Ruby: Files in GEM_HOME/bin: {bin_files}")
                except Exception as e:
                    logger.debug(f"Ruby: Could not list GEM_HOME/bin: {e}")
            else:
                logger.debug(f"Ruby: GEM_HOME/bin directory does not exist: {gem_home_bin_dir}")
        else:
            logger.debug("Ruby: GEM_HOME is not set")
        
        # Also try system gem locations via Ruby
        try:
            import subprocess as sp
            gem_env_result = sp.run(
                ["ruby", "-e", "puts Gem.user_dir"],
                capture_output=True,
                text=True,
                encoding='utf-8',
                errors='replace',
                timeout=2
            )
            if gem_env_result.returncode == 0:
                gem_dir = gem_env_result.stdout.strip()
                if gem_dir:
                    gem_paths.append(os.path.join(gem_dir, "bin", "swhid"))
        except Exception:
            pass
        
        # Try to find swhid in gem-specific locations first
        for pattern in gem_paths:
            logger.debug(f"Ruby: Checking pattern: {pattern}")
            # Normalize the pattern path for Windows compatibility
            pattern_normalized = os.path.normpath(pattern)
            
            # Try glob first (for wildcard patterns)
            matches = glob.glob(pattern_normalized)
            
            # Also try direct path check (for exact paths, especially on Windows)
            if os.path.isfile(pattern_normalized):
                matches.append(pattern_normalized)
            
            # On Windows, also try .bat and .cmd extensions
            if is_windows:
                bat_pattern = pattern_normalized + ".bat"
                cmd_pattern = pattern_normalized + ".cmd"
                matches.extend(glob.glob(bat_pattern))
                matches.extend(glob.glob(cmd_pattern))
                # Direct path checks for .bat/.cmd
                if os.path.isfile(bat_pattern):
                    matches.append(bat_pattern)
                if os.path.isfile(cmd_pattern):
                    matches.append(cmd_pattern)
            
            # Remove duplicates while preserving order
            seen = set()
            unique_matches = []
            for match in matches:
                normalized_match = os.path.normpath(match)
                if normalized_match not in seen:
                    seen.add(normalized_match)
                    unique_matches.append(normalized_match)
            
            # On Windows, prefer .bat/.cmd files over Unix scripts
            if is_windows:
                # Sort matches to prefer .bat/.cmd files
                def sort_key(cmd):
                    if cmd.endswith('.bat'):
                        return 0
                    elif cmd.endswith('.cmd'):
                        return 1
                    elif cmd.endswith('.exe'):
                        return 2
                    else:
                        return 3
                unique_matches.sort(key=sort_key)
            
            for swhid_cmd in unique_matches:
                logger.debug(f"Ruby: Checking candidate: {swhid_cmd}")
                # On Windows, check if file exists (os.X_OK may not work for .bat files)
                if os.path.isfile(swhid_cmd):
                    logger.debug(f"Ruby: File exists: {swhid_cmd}")
                    # On Windows, .bat/.cmd files are always "executable"
                    if is_windows or os.access(swhid_cmd, os.X_OK):
                        logger.debug(f"Ruby: File is accessible, verifying it's Ruby gem...")
                        # Verify it's the Ruby gem by checking if it supports 'snapshot' command
                        # (Rust version doesn't support snapshot yet)
                        # On Windows, if we have a .bat file, we can be more confident it's the Ruby gem
                        if is_windows and swhid_cmd.endswith(('.bat', '.cmd')):
                            logger.info(f"Ruby: Found Ruby gem .bat wrapper at: {swhid_cmd}")
                            self._swhid_path = swhid_cmd
                            return swhid_cmd
                        
                        try:
                            result = subprocess.run(
                                [swhid_cmd, "snapshot", "--help"],
                                capture_output=True,
                                text=True,
                                timeout=2
                            )
                            logger.debug(f"Ruby: snapshot --help result: returncode={result.returncode}, stdout={result.stdout[:100]}, stderr={result.stderr[:100]}")
                            if result.returncode == 0 or "snapshot" in result.stdout or "snapshot" in result.stderr:
                                logger.info(f"Ruby: Found Ruby gem swhid at: {swhid_cmd}")
                                self._swhid_path = swhid_cmd
                                return swhid_cmd
                            else:
                                logger.debug(f"Ruby: Binary doesn't support snapshot command, skipping")
                        except Exception as e:
                            logger.debug(f"Ruby: Exception verifying binary: {e}")
                            # On Windows, if verification fails but we have a .bat file, assume it's Ruby gem
                            if is_windows and swhid_cmd.endswith(('.bat', '.cmd')):
                                logger.info(f"Ruby: Found Ruby gem .bat wrapper (verification failed, but .bat indicates Ruby gem): {swhid_cmd}")
                                self._swhid_path = swhid_cmd
                                return swhid_cmd
                            # If check fails, assume it's the Ruby gem (better than nothing)
                            logger.warning(f"Ruby: Could not verify snapshot support, assuming Ruby gem: {swhid_cmd}")
                            self._swhid_path = swhid_cmd
                            return swhid_cmd
                else:
                    logger.debug(f"Ruby: File does not exist: {swhid_cmd}")
        
        # Fallback: try to find swhid command in PATH (may be Rust binary)
        logger.debug("Ruby: Checking PATH for swhid command")
        path_env = os.environ.get("PATH", "")
        logger.debug(f"Ruby: PATH contains {len(path_env.split(os.pathsep))} entries")
        if is_windows:
            for ext in ["", ".bat", ".cmd"]:
                swhid_name = "swhid" + ext
                swhid_path = shutil.which(swhid_name)
                if swhid_path:
                    logger.debug(f"Ruby: Found {swhid_name} in PATH: {swhid_path}")
                    # Verify it supports snapshot (Ruby gem) vs not (Rust binary)
                    try:
                        result = subprocess.run(
                            [swhid_path, "snapshot", "--help"],
                            capture_output=True,
                            text=True,
                            timeout=2
                        )
                        logger.debug(f"Ruby: PATH binary snapshot check: returncode={result.returncode}")
                        if result.returncode == 0 or "snapshot" in result.stdout or "snapshot" in result.stderr:
                            logger.info(f"Ruby: Found Ruby gem swhid in PATH: {swhid_path}")
                            self._swhid_path = swhid_path
                            return swhid_path
                        else:
                            logger.debug(f"Ruby: PATH binary doesn't support snapshot, likely Rust binary")
                    except Exception as e:
                        logger.debug(f"Ruby: Exception checking PATH binary: {e}")
        else:
            swhid_path = shutil.which("swhid")
            if swhid_path:
                logger.debug(f"Ruby: Found swhid in PATH: {swhid_path}")
                # Verify it supports snapshot (Ruby gem) vs not (Rust binary)
                try:
                    result = subprocess.run(
                        [swhid_path, "snapshot", "--help"],
                        capture_output=True,
                        text=True,
                        timeout=2
                    )
                    if result.returncode == 0 or "snapshot" in result.stdout or "snapshot" in result.stderr:
                        logger.info(f"Ruby: Found Ruby gem swhid in PATH: {swhid_path}")
                        self._swhid_path = swhid_path
                        return swhid_path
                except Exception as e:
                    logger.debug(f"Ruby: Exception checking PATH binary: {e}")
        
        logger.warning("Ruby: Could not find swhid binary in any location")
        return None

    def get_info(self) -> ImplementationInfo:
        """Return implementation metadata."""
        return ImplementationInfo(
            name="ruby",
            version="0.3.1",
            language="ruby",
            description="Ruby SWHID implementation via swhid gem",
            test_command="swhid --help",
            dependencies=["swhid"]
        )

    def is_available(self) -> bool:
        """Check if Ruby implementation is available."""
        logger.debug("Ruby: Checking availability")
        swhid_path = self._find_swhid_path()
        if not swhid_path:
            logger.warning("Ruby: Implementation not available - swhid binary not found")
            return False
        
        logger.debug(f"Ruby: Testing binary at: {swhid_path}")
        
        # On Windows, prefer .bat file if we found the Unix script
        is_windows = platform.system() == "Windows"
        if is_windows and not swhid_path.endswith(('.bat', '.cmd', '.exe')):
            bat_path = swhid_path + '.bat'
            if os.path.isfile(bat_path):
                logger.debug(f"Ruby: Using .bat wrapper on Windows: {bat_path}")
                swhid_path = bat_path
        
        # Test that the command actually works
        try:
            logger.debug(f"Ruby: Running test command: {swhid_path} help")
            result = subprocess.run(
                [swhid_path, "help"],
                capture_output=True,
                text=True,
                encoding='utf-8',
                errors='replace',
                timeout=5
            )
            logger.debug(f"Ruby: Test command result: returncode={result.returncode}, stdout={result.stdout[:100]}, stderr={result.stderr[:100]}")
            if result.returncode == 0:
                logger.info(f"Ruby: Implementation is available at: {swhid_path}")
                return True
            else:
                logger.warning(f"Ruby: Test command failed with returncode {result.returncode}")
                return False
        except subprocess.TimeoutExpired:
            logger.warning("Ruby: Test command timed out")
            return False
        except Exception as e:
            logger.warning(f"Ruby: Exception testing binary: {e}")
            return False

    def get_capabilities(self) -> ImplementationCapabilities:
        """Return implementation capabilities."""
        return ImplementationCapabilities(
            supported_types=["cnt", "dir", "rev", "rel", "snp"],
            supported_qualifiers=["origin", "visit", "anchor", "path", "lines", "bytes"],
            api_version="1.0",
            max_payload_size_mb=1000,
            supports_unicode=True,
            supports_percent_encoding=True
        )

    def compute_swhid(self, payload_path: str, obj_type: Optional[str] = None,
                     commit: Optional[str] = None, tag: Optional[str] = None,
                     version: Optional[int] = None, hash_algo: Optional[str] = None) -> str:
        """Compute SWHID for a payload using the Ruby implementation.
        
        Note: version and hash_algo parameters are accepted for API compatibility
        but are ignored as the Ruby implementation only supports v1/SHA1.
        """
        
        # Get the swhid command path
        swhid_path = self._find_swhid_path()
        if not swhid_path:
            raise RuntimeError("Ruby implementation not found (swhid gem not installed)")
        
        # Build the command
        cmd = [swhid_path]

        # Map object types to swhid CLI commands
        if obj_type == "content" or obj_type == "cnt":
            cmd.append("content")
        elif obj_type == "directory" or obj_type == "dir":
            cmd.append("directory")
        elif obj_type == "revision" or obj_type == "rev":
            cmd.extend(["revision", payload_path])
            if commit:
                cmd.append(commit)
        elif obj_type == "release" or obj_type == "rel":
            cmd.extend(["release", payload_path])
            if tag:
                cmd.append(tag)
        elif obj_type == "snapshot" or obj_type == "snp":
            cmd.extend(["snapshot", payload_path])
        elif obj_type is None or obj_type == "auto":
            # Auto-detect based on path
            if os.path.isfile(payload_path):
                cmd.append("content")
            elif os.path.isdir(payload_path):
                # Check if it's a git repository
                if os.path.isdir(os.path.join(payload_path, ".git")):
                    cmd.extend(["snapshot", payload_path])
                else:
                    cmd.append("directory")
            else:
                raise ValueError(f"Cannot determine object type for {payload_path}")
        else:
            raise NotImplementedError(f"Ruby implementation doesn't support {obj_type} object type")

        # For content type, read from stdin
        if cmd[-1] == "content":
            try:
                # Read file and pipe to stdin
                with open(payload_path, 'rb') as f:
                    content = f.read()

                result = subprocess.run(
                    cmd,
                    input=content,
                    capture_output=True,
                    timeout=30
                )

                if result.returncode != 0:
                    stderr = result.stderr.decode('utf-8', errors='replace')
                    raise RuntimeError(f"Ruby implementation failed: {stderr}")

                # Parse the output
                output = result.stdout.decode('utf-8', errors='replace').strip()
                if not output:
                    raise RuntimeError("No output from Ruby implementation")

                if not output.startswith("swh:"):
                    raise RuntimeError(f"Invalid SWHID format: {output}")

                return output

            except subprocess.TimeoutExpired:
                raise RuntimeError("Ruby implementation timed out")
            except FileNotFoundError as e:
                raise RuntimeError(f"File not found: {e}")
            except Exception as e:
                raise RuntimeError(f"Error running Ruby implementation: {e}")

        # For directory and git types, pass path as argument
        elif cmd[1] in ["directory", "revision", "release", "snapshot"]:
            if cmd[1] == "directory":
                # On Windows, we need to preserve file permissions before calling the tool
                # Create a temporary copy with correct permissions
                payload_path = self._ensure_permissions_preserved(payload_path)
                cmd.append(payload_path)

            try:
                result = subprocess.run(
                    cmd,
                    capture_output=True,
                    text=True,
                    encoding='utf-8',
                    errors='replace',
                    timeout=30
                )

                if result.returncode != 0:
                    raise RuntimeError(f"Ruby implementation failed: {result.stderr}")

                # Parse the output
                output = result.stdout.strip()
                if not output:
                    raise RuntimeError("No output from Ruby implementation")

                if not output.startswith("swh:"):
                    raise RuntimeError(f"Invalid SWHID format: {output}")

                return output

            except subprocess.TimeoutExpired:
                raise RuntimeError("Ruby implementation timed out")
            except FileNotFoundError:
                raise RuntimeError("Ruby implementation not found (swhid gem not installed)")
            except Exception as e:
                raise RuntimeError(f"Error running Ruby implementation: {e}")
            finally:
                # Cleanup temporary directories if created
                self._cleanup_temp_dirs()
    
    def _ensure_permissions_preserved(self, source_path: str) -> str:
        """Ensure file permissions are preserved for external tools.
        
        On Windows, files lose executable bits when copied. This method creates
        a temporary copy with correct permissions set, which external tools
        (like the Ruby swhid gem) can read correctly.
        
        Args:
            source_path: Path to source file or directory
        
        Returns:
            Path to use (may be temporary copy on Windows, or original on Unix)
        """
        import stat
        import tempfile
        import shutil
        import platform
        
        # On Unix-like systems, permissions are usually preserved
        # Only create temp copy on Windows
        if platform.system() != 'Windows':
            return source_path
        
        # Read source permissions
        # On Windows, check Git index first as filesystem may not preserve executable bits
        source_permissions = {}
        
        # On Windows, try to read permissions from Git index first
        # This is more reliable than filesystem permissions
        try:
            # Get absolute path to source_path
            abs_source_path = os.path.abspath(source_path)
            # Get repository root (walk up to find .git)
            repo_root = abs_source_path
            if os.path.isdir(repo_root):
                check_path = repo_root
            else:
                check_path = os.path.dirname(repo_root)
            
            while check_path != os.path.dirname(check_path):
                if os.path.exists(os.path.join(check_path, '.git')):
                    repo_root = check_path
                    break
                check_path = os.path.dirname(check_path)
            else:
                repo_root = None
            
            # If we found a repo, check Git index for permissions
            if repo_root:
                if os.path.isdir(source_path):
                    for root, dirs, files in os.walk(source_path):
                        for file in files:
                            file_path = os.path.join(root, file)
                            rel_path = os.path.relpath(file_path, source_path)
                            # Normalize path separators to forward slashes for cross-platform consistency
                            rel_path = rel_path.replace(os.sep, '/')
                            
                            # Get path relative to repo root
                            try:
                                repo_rel_path = os.path.relpath(file_path, repo_root)
                                # Normalize for Git command (Git uses forward slashes)
                                repo_rel_path = repo_rel_path.replace(os.sep, '/')
                                # Check Git index
                                result = subprocess.run(
                                    ['git', 'ls-files', '--stage', repo_rel_path],
                                    cwd=repo_root,
                                    capture_output=True,
                                    text=True,
                                    encoding='utf-8',
                                    errors='replace',
                                    timeout=2
                                )
                                if result.returncode == 0 and result.stdout.strip():
                                    # Format: <mode> <sha> <stage> <path>
                                    parts = result.stdout.strip().split()
                                    if parts:
                                        git_mode = parts[0]
                                        # Mode is octal string, e.g., '100755' for executable
                                        is_executable = git_mode.endswith('755')
                                        source_permissions[rel_path] = is_executable
                                        continue
                            except (subprocess.TimeoutExpired, subprocess.CalledProcessError, ValueError):
                                pass
                elif os.path.isfile(source_path):
                    try:
                        repo_rel_path = os.path.relpath(source_path, repo_root)
                        result = subprocess.run(
                            ['git', 'ls-files', '--stage', repo_rel_path],
                            cwd=repo_root,
                            capture_output=True,
                            text=True,
                            encoding='utf-8',
                            errors='replace',
                            timeout=2
                        )
                        if result.returncode == 0 and result.stdout.strip():
                            parts = result.stdout.strip().split()
                            if parts:
                                git_mode = parts[0]
                                is_executable = git_mode.endswith('755')
                                source_permissions['.'] = is_executable
                    except (subprocess.TimeoutExpired, subprocess.CalledProcessError, ValueError):
                        pass
        except Exception:
            # If Git check fails, fall back to filesystem
            pass
        
        # Fall back to filesystem permissions (works on Unix, or if Git check failed)
        if os.path.isdir(source_path):
            for root, dirs, files in os.walk(source_path):
                for file in files:
                    file_path = os.path.join(root, file)
                    rel_path = os.path.relpath(file_path, source_path)
                    # Normalize path separators to forward slashes for cross-platform consistency
                    rel_path = rel_path.replace(os.sep, '/')
                    
                    # Skip if we already got permission from Git index
                    if rel_path in source_permissions:
                        continue
                    
                    try:
                        stat_info = os.stat(file_path)
                        is_executable = bool(stat_info.st_mode & stat.S_IEXEC)
                        source_permissions[rel_path] = is_executable
                    except OSError:
                        source_permissions[rel_path] = False
        elif os.path.isfile(source_path):
            # Skip if we already got permission from Git index
            if '.' not in source_permissions:
                try:
                    stat_info = os.stat(source_path)
                    is_executable = bool(stat_info.st_mode & stat.S_IEXEC)
                    source_permissions['.'] = is_executable  # Single file, use '.' as key
                except OSError:
                    source_permissions['.'] = False
        
        # If no executable files found, no need for temp copy
        if not any(source_permissions.values()):
            return source_path
        
        # Create temporary copy with permissions
        temp_dir = tempfile.mkdtemp(prefix="swhid-ruby-")
        self._temp_dirs.append(temp_dir)
        
        if os.path.isdir(source_path):
            # Copy directory
            temp_path = os.path.join(temp_dir, os.path.basename(source_path) or "dir")
            shutil.copytree(source_path, temp_path, symlinks=True)
            
            # Apply permissions
            for rel_path, is_executable in source_permissions.items():
                target_file = os.path.join(temp_path, rel_path)
                if os.path.exists(target_file):
                    try:
                        current_stat = os.stat(target_file)
                        if is_executable:
                            os.chmod(target_file, current_stat.st_mode | stat.S_IEXEC)
                        else:
                            os.chmod(target_file, current_stat.st_mode & ~stat.S_IEXEC)
                    except OSError:
                        # On Windows, chmod might not work - that's okay
                        pass
            
            return temp_path
        else:
            # Copy single file
            temp_path = os.path.join(temp_dir, os.path.basename(source_path))
            shutil.copy2(source_path, temp_path)
            
            # Apply permission
            if source_permissions.get('.', False):
                try:
                    current_stat = os.stat(temp_path)
                    os.chmod(temp_path, current_stat.st_mode | stat.S_IEXEC)
                except OSError:
                    pass
            
            return temp_path
    
    def _cleanup_temp_dirs(self):
        """Clean up temporary directories created for permission preservation."""
        import shutil
        for temp_dir in self._temp_dirs:
            try:
                if os.path.exists(temp_dir):
                    shutil.rmtree(temp_dir)
            except Exception:
                pass
        self._temp_dirs.clear()

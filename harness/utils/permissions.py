"""
Permission utilities for the SWHID Testing Harness.

This module provides shared utilities for reading and preserving file permissions,
especially for cross-platform compatibility (Windows vs Unix).
"""

import os
import stat
import subprocess
import platform
from typing import Dict, Optional, Tuple
import logging

logger = logging.getLogger(__name__)


def get_source_permissions(source_path: str) -> Dict[str, bool]:
    """
    Read file permissions from source path.
    
    On Windows, tries to read from Git index first (more reliable),
    then falls back to filesystem permissions.
    On Unix, reads from filesystem.
    
    Args:
        source_path: Path to source file or directory
        
    Returns:
        Dictionary mapping relative paths to executable flags
        (use '.' for single files)
    """
    source_permissions: Dict[str, bool] = {}
    
    # On Windows, try to read permissions from Git index first
    # This is more reliable than filesystem permissions
    if platform.system() == 'Windows':
        try:
            # Get absolute path to source_path
            abs_source_path = os.path.abspath(source_path)
            # Get repository root (walk up to find .git)
            repo_root = _find_git_repo_root(abs_source_path)
            
            # If we found a repo, check Git index for permissions
            if repo_root:
                if os.path.isdir(source_path):
                    source_permissions.update(_read_permissions_from_git_index_dir(source_path, repo_root))
                elif os.path.isfile(source_path):
                    source_permissions.update(_read_permissions_from_git_index_file(source_path, repo_root))
        except Exception:
            # If Git check fails, fall back to filesystem
            logger.debug("Failed to read permissions from Git index, falling back to filesystem")
    
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
    
    return source_permissions


def _find_git_repo_root(path: str) -> Optional[str]:
    """
    Find Git repository root by walking up from path.
    
    Args:
        path: Starting path (file or directory)
        
    Returns:
        Repository root path if found, None otherwise
    """
    abs_path = os.path.abspath(path)
    if os.path.isdir(abs_path):
        check_path = abs_path
    else:
        check_path = os.path.dirname(abs_path)
    
    while check_path != os.path.dirname(check_path):
        if os.path.exists(os.path.join(check_path, '.git')):
            return check_path
        check_path = os.path.dirname(check_path)
    
    return None


def _read_permissions_from_git_index_dir(source_path: str, repo_root: str) -> Dict[str, bool]:
    """
    Read permissions from Git index for a directory.
    
    Args:
        source_path: Path to source directory
        repo_root: Git repository root
        
    Returns:
        Dictionary mapping relative paths to executable flags
    """
    permissions: Dict[str, bool] = {}
    
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
                        permissions[rel_path] = is_executable
            except (subprocess.TimeoutExpired, subprocess.CalledProcessError, ValueError):
                pass
    
    return permissions


def _read_permissions_from_git_index_file(source_path: str, repo_root: str) -> Dict[str, bool]:
    """
    Read permissions from Git index for a single file.
    
    Args:
        source_path: Path to source file
        repo_root: Git repository root
        
    Returns:
        Dictionary with '.' key mapping to executable flag
    """
    permissions: Dict[str, bool] = {}
    
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
                permissions['.'] = is_executable
    except (subprocess.TimeoutExpired, subprocess.CalledProcessError, ValueError):
        pass
    
    return permissions


def create_git_repo_with_permissions(
    source_path: str,
    source_permissions: Dict[str, bool],
    temp_dir: str,
    target_subdir: str = "target"
) -> Tuple[str, bool]:
    """
    Create a temporary Git repository with permissions set in the Git index.
    
    This is used on Windows where filesystem permissions are not reliable.
    The Git index preserves executable permissions which external tools can read.
    
    Args:
        source_path: Path to source file or directory
        source_permissions: Dictionary mapping relative paths to executable flags
        temp_dir: Temporary directory for the Git repository
        target_subdir: Subdirectory name within repo to place files (default: "target")
        
    Returns:
        Tuple of (path_to_use, success_flag)
        - path_to_use: Path to the target subdirectory or file within the Git repo
        - success_flag: True if Git repo was created successfully
    """
    import shutil
    
    repo_path = os.path.join(temp_dir, "repo")
    os.makedirs(repo_path, exist_ok=True)
    
    # Initialize Git repository
    try:
        subprocess.run(
            ["git", "init"],
            cwd=repo_path,
            check=True,
            capture_output=True,
            encoding='utf-8',
            errors='replace'
        )
    except subprocess.CalledProcessError:
        return source_path, False
    
    # Configure Git for SWHID testing (preserve line endings and permissions)
    git_configs = [
        ("core.autocrlf", "false"),
        ("core.filemode", "true"),
        ("core.precomposeunicode", "false"),
    ]
    
    for config_key, config_value in git_configs:
        try:
            subprocess.run(
                ["git", "config", config_key, config_value],
                cwd=repo_path,
                check=True,
                capture_output=True,
                encoding='utf-8',
                errors='replace'
            )
        except subprocess.CalledProcessError:
            logger.warning(f"Failed to set Git config {config_key}={config_value}")
    
    # Copy directory or file to target subdirectory
    target_subdir_path = os.path.join(repo_path, target_subdir)
    os.makedirs(target_subdir_path, exist_ok=True)
    
    if os.path.isdir(source_path):
        # Copy directory contents to target subdirectory
        # Preserve symlinks (important for mixed_types test)
        for item in os.listdir(source_path):
            src_item = os.path.join(source_path, item)
            dst_item = os.path.join(target_subdir_path, item)
            if os.path.islink(src_item):
                # Preserve symlinks by copying the symlink itself, not the target
                link_target = os.readlink(src_item)
                os.symlink(link_target, dst_item)
            elif os.path.isdir(src_item):
                shutil.copytree(src_item, dst_item, symlinks=True)
            else:
                shutil.copy2(src_item, dst_item)
        
        # Add all files to Git index (from target subdirectory)
        try:
            subprocess.run(
                ["git", "add", target_subdir],
                cwd=repo_path,
                check=True,
                capture_output=True,
                encoding='utf-8',
                errors='replace'
            )
        except subprocess.CalledProcessError:
            return source_path, False
        
        # Apply executable permissions to Git index
        # Paths must be relative to repo root (include target_subdir prefix)
        for rel_path, is_executable in source_permissions.items():
            if is_executable:
                # Path relative to source directory, prepend target_subdir for Git index
                git_path = os.path.join(target_subdir, rel_path).replace(os.sep, '/')
                # Verify file exists in repo before trying to set permission
                file_path = os.path.join(repo_path, git_path)
                if os.path.exists(file_path):
                    try:
                        subprocess.run(
                            ["git", "update-index", "--chmod=+x", git_path],
                            cwd=repo_path,
                            check=True,
                            capture_output=True,
                            encoding='utf-8',
                            errors='replace'
                        )
                        logger.debug(f"Set executable permission for {git_path} in Git index")
                    except subprocess.CalledProcessError as e:
                        logger.warning(f"Failed to set executable permission for {git_path}: {e.stderr}")
        
        # Refresh the Git index to ensure all changes are written to disk
        try:
            subprocess.run(
                ["git", "update-index", "--refresh"],
                cwd=repo_path,
                check=True,
                capture_output=True,
                encoding='utf-8',
                errors='replace'
            )
            logger.debug("Refreshed Git index")
        except subprocess.CalledProcessError:
            logger.debug("Git index refresh failed (non-critical)")
        
        return target_subdir_path, True
    else:
        # Copy single file
        target_file = os.path.join(target_subdir_path, os.path.basename(source_path))
        shutil.copy2(source_path, target_file)
        
        # Add to Git index
        file_name = os.path.join(target_subdir, os.path.basename(source_path)).replace(os.sep, '/')
        try:
            subprocess.run(
                ["git", "add", file_name],
                cwd=repo_path,
                check=True,
                capture_output=True,
                encoding='utf-8',
                errors='replace'
            )
        except subprocess.CalledProcessError:
            return source_path, False
        
        # Apply executable permission if needed
        if source_permissions.get('.', False):
            try:
                subprocess.run(
                    ["git", "update-index", "--chmod=+x", file_name],
                    cwd=repo_path,
                    check=True,
                    capture_output=True,
                    encoding='utf-8',
                    errors='replace'
                )
            except subprocess.CalledProcessError:
                pass
        
        return target_file, True


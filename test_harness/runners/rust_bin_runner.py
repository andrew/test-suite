#!/usr/bin/env python3
"""
Pre-compiled Rust Binary SWHID Runner

This runner uses a pre-compiled Rust binary to avoid
subprocess compilation overhead for accurate performance testing.
"""

import os
import sys
import subprocess
from pathlib import Path
from typing import Optional


def compute_swhid(payload_path: str, obj_type: Optional[str] = None) -> str:
    """
    Compute SWHID using pre-compiled Rust binary.
    
    Args:
        payload_path: Path to the payload file/directory
        obj_type: Object type (content, directory, etc.)
    
    Returns:
        SWHID string in format swh:1:obj_type:hash
    """
    payload_path = os.path.abspath(payload_path)
    
    if not os.path.exists(payload_path):
        raise FileNotFoundError(f"Payload not found: {payload_path}")
    
    # Auto-detect object type if not provided
    if obj_type is None:
        obj_type = detect_object_type(payload_path)
    
    # Find the compiled binary
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    # The project root should be the swhid-rs directory, not the parent
    if not os.path.exists(os.path.join(project_root, "Cargo.toml")):
        # If we're not in the right directory, go up one more level
        project_root = os.path.dirname(project_root)
    binary_path = os.path.join(project_root, "target", "release", "swhid-cli")
    
    if not os.path.exists(binary_path):
        # Try debug build
        binary_path = os.path.join(project_root, "target", "debug", "swhid-cli")
        if not os.path.exists(binary_path):
            raise RuntimeError("Rust binary not found. Run 'cargo build --release' first")
    
    # Build the command
    cmd = [binary_path]
    
    # Add object type if specified
    if obj_type:
        cmd.extend(["--obj-type", obj_type])
    
    # Add the payload path
    cmd.append(payload_path)
    
    # Run the command
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode != 0:
            raise RuntimeError(f"Rust binary failed: {result.stderr}")
        
        # Parse the output
        output = result.stdout.strip()
        if not output:
            raise RuntimeError("No output from Rust binary")
        
        # The output format is: SWHID\tfilename (optional)
        # We want just the SWHID part
        swhid = output.split('\t')[0].strip()
        
        if not swhid.startswith("swh:"):
            raise RuntimeError(f"Invalid SWHID format: {swhid}")
        
        return swhid
        
    except subprocess.TimeoutExpired:
        raise RuntimeError("Rust binary timed out")
    except FileNotFoundError:
        raise RuntimeError("Rust binary not found")
    except Exception as e:
        raise RuntimeError(f"Error running Rust binary: {e}")


def detect_object_type(payload_path: str) -> str:
    """Detect object type from payload path."""
    if os.path.isfile(payload_path):
        return "content"
    elif os.path.isdir(payload_path):
        return "directory"
    else:
        raise ValueError(f"Cannot detect object type for: {payload_path}")


def compute_swhid_detailed(payload_path: str, obj_type: Optional[str] = None, 
                          archive: bool = False) -> str:
    """
    Compute SWHID with detailed parameters (for compatibility with other runners).
    
    Args:
        payload_path: Path to the payload
        obj_type: Object type
        archive: Whether this is an archive
    
    Returns:
        SWHID string
    """
    if archive:
        # Add archive flag
        payload_path = os.path.abspath(payload_path)
        project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        binary_path = os.path.join(project_root, "target", "release", "swhid-cli")
        
        if not os.path.exists(binary_path):
            binary_path = os.path.join(project_root, "target", "debug", "swhid-cli")
        
        cmd = [binary_path, "--archive", payload_path]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        if result.returncode != 0:
            raise RuntimeError(f"Rust binary failed: {result.stderr}")
        
        output = result.stdout.strip()
        swhid = output.split('\t')[0].strip()
        return swhid
    
    return compute_swhid(payload_path, obj_type)


def compute_swhid_auto(payload_path: str) -> str:
    """Auto-detect object type and compute SWHID."""
    return compute_swhid(payload_path)


def compute_swhid_simple(payload_path: str, obj_type: str = None) -> str:
    """Compute SWHID with explicit object type."""
    return compute_swhid(payload_path, obj_type)


if __name__ == "__main__":
    # Simple CLI interface for testing
    if len(sys.argv) < 2:
        print("Usage: python rust_bin_runner.py <payload_path> [obj_type]")
        sys.exit(1)
    
    payload_path = sys.argv[1]
    obj_type = sys.argv[2] if len(sys.argv) > 2 else None
    
    try:
        swhid = compute_swhid(payload_path, obj_type)
        print(swhid)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1) 
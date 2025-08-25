#!/usr/bin/env python3
"""
Direct Rust Library SWHID Runner

This runner uses the Rust implementation directly as a library,
bypassing subprocess overhead for accurate performance testing.
"""

import os
import sys
from pathlib import Path
from typing import Optional

# Add the project root to Python path to import Rust library
project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, project_root)

try:
    import swhid
    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False


def compute_swhid(payload_path: str, obj_type: Optional[str] = None) -> str:
    """
    Compute SWHID using Rust library directly.
    
    Args:
        payload_path: Path to the payload file/directory
        obj_type: Object type (content, directory, etc.)
    
    Returns:
        SWHID string in format swh:1:obj_type:hash
    """
    if not RUST_AVAILABLE:
        raise RuntimeError("Rust library not available - run 'cargo build' first")
    
    payload_path = os.path.abspath(payload_path)
    
    if not os.path.exists(payload_path):
        raise FileNotFoundError(f"Payload not found: {payload_path}")
    
    # Auto-detect object type if not provided
    if obj_type is None:
        obj_type = detect_object_type(payload_path)
    
    try:
        # Create SWHID computer
        computer = swhid.SwhidComputer()
        
        if obj_type == "content":
            swhid_obj = computer.compute_content_swhid(payload_path)
        elif obj_type == "directory":
            swhid_obj = computer.compute_directory_swhid(payload_path)
        elif obj_type == "snapshot":
            swhid_obj = computer.compute_snapshot_swhid(payload_path)
        else:
            raise ValueError(f"Unsupported object type: {obj_type}")
        
        return str(swhid_obj)
        
    except Exception as e:
        raise RuntimeError(f"Failed to compute Rust SWHID: {e}")


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
        # Handle archive processing
        computer = swhid.SwhidComputer()
        return str(computer.compute_archive_directory_swhid(payload_path))
    
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
        print("Usage: python rust_lib_runner.py <payload_path> [obj_type]")
        sys.exit(1)
    
    payload_path = sys.argv[1]
    obj_type = sys.argv[2] if len(sys.argv) > 2 else None
    
    try:
        swhid = compute_swhid(payload_path, obj_type)
        print(swhid)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1) 
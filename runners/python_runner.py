import subprocess
from typing import Optional

def compute_swhid(payload_path: str, obj_type: Optional[str] = None, 
                  archive: bool = False) -> str:
    """
    Compute SWHID for a payload using the Python implementation.
    
    Args:
        payload_path: Path to the payload file/directory
        obj_type: Object type (content, directory, snapshot, auto)
        archive: Whether to treat file as archive (not supported in Python impl)
    
    Returns:
        SWHID string
    """
    # Build the command
    cmd = ["python", "-m", "swh.model.cli"]
    
    # Add object type if specified
    if obj_type and obj_type != "auto":
        cmd.extend(["--type", obj_type])
    
    # Add archive flag if requested (note: Python impl may not support this)
    if archive:
        cmd.append("--archive")
    
    # Add the payload path
    cmd.append(payload_path)
    print("COMMAND:", " ".join(cmd))  # Debug print
    
    # Run the command
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode != 0:
            raise RuntimeError(f"Python implementation failed: {result.stderr}")
        
        # Parse the output
        output = result.stdout.strip()
        if not output:
            raise RuntimeError("No output from Python implementation")
        
        # The output format is: SWHID\tfilename (optional)
        # We want just the SWHID part
        swhid = output.split('\t')[0].strip()
        
        if not swhid.startswith("swh:"):
            raise RuntimeError(f"Invalid SWHID format: {swhid}")
        
        return swhid
        
    except subprocess.TimeoutExpired:
        raise RuntimeError("Python implementation timed out")
    except FileNotFoundError:
        raise RuntimeError("Python implementation not found (swh.model not available)")
    except Exception as e:
        raise RuntimeError(f"Error running Python implementation: {e}") 
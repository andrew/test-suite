#!/usr/bin/env python3
"""
Performance comparison test for SWHID implementations.

This script compares the performance of all implementations
using the swh-model directory as a test payload.
"""

import os
import sys
import time
import statistics
from pathlib import Path
from typing import Dict, List, Tuple
import yaml
import importlib.util

# Add the current directory to the path so we can import runners
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

def load_config():
    """Load the harness configuration."""
    config_path = os.path.join(os.path.dirname(__file__), "config.yaml")
    with open(config_path, 'r') as f:
        return yaml.safe_load(f)

def load_runner(runner_path: str):
    """Load a runner module dynamically."""
    spec = importlib.util.spec_from_file_location("runner", runner_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def test_implementation(impl_name: str, runner_path: str, test_path: str, iterations: int = 5) -> Dict:
    """Test a single implementation and return performance metrics."""
    print(f"Testing {impl_name}...")
    
    try:
        runner = load_runner(runner_path)
        times = []
        
        for i in range(iterations):
            start_time = time.time()
            try:
                # Use the appropriate function based on runner type
                if impl_name in ['git', 'git-cmd', 'pygit2']:
                    swhid = runner.compute_swhid_simple(test_path, "directory")
                else:
                    swhid = runner.compute_swhid_simple(test_path)
                end_time = time.time()
                duration = end_time - start_time
                times.append(duration)
                print(f"  Iteration {i+1}: {duration:.3f}s -> {swhid}")
            except Exception as e:
                print(f"  Iteration {i+1}: ERROR - {e}")
                return {
                    "name": impl_name,
                    "status": "error",
                    "error": str(e),
                    "times": [],
                    "swhid": None
                }
        
        return {
            "name": impl_name,
            "status": "success",
            "times": times,
            "mean": statistics.mean(times),
            "median": statistics.median(times),
            "min": min(times),
            "max": max(times),
            "std": statistics.stdev(times) if len(times) > 1 else 0,
            "swhid": swhid
        }
        
    except Exception as e:
        print(f"  Failed to load runner: {e}")
        return {
            "name": impl_name,
            "status": "error",
            "error": str(e),
            "times": [],
            "swhid": None
        }

def print_results(results: List[Dict]):
    """Print performance comparison results."""
    print("\n" + "="*80)
    print("PERFORMANCE COMPARISON RESULTS")
    print("="*80)
    
    # Filter successful results
    successful = [r for r in results if r["status"] == "success"]
    failed = [r for r in results if r["status"] == "error"]
    
    if successful:
        print(f"\nSuccessful implementations ({len(successful)}):")
        print("-" * 80)
        
        # Sort by mean time (fastest first)
        successful.sort(key=lambda x: x["mean"])
        
        for i, result in enumerate(successful, 1):
            print(f"{i:2d}. {result['name']:<20} "
                  f"Mean: {result['mean']:6.3f}s "
                  f"Median: {result['median']:6.3f}s "
                  f"Min: {result['min']:6.3f}s "
                  f"Max: {result['max']:6.3f}s "
                  f"Std: {result['std']:6.3f}s")
            print(f"    SWHID: {result['swhid']}")
            print()
    
    if failed:
        print(f"\nFailed implementations ({len(failed)}):")
        print("-" * 80)
        for result in failed:
            print(f"❌ {result['name']}: {result['error']}")
    
    # Summary statistics
    if successful:
        print("\nSUMMARY:")
        print("-" * 80)
        fastest = successful[0]
        slowest = successful[-1]
        speedup = slowest["mean"] / fastest["mean"]
        
        print(f"Fastest:  {fastest['name']} ({fastest['mean']:.3f}s)")
        print(f"Slowest:  {slowest['name']} ({slowest['mean']:.3f}s)")
        print(f"Speedup:  {speedup:.1f}x")
        
        # Calculate relative performance
        print(f"\nRelative Performance (normalized to fastest):")
        for result in successful:
            relative = result["mean"] / fastest["mean"]
            print(f"  {result['name']:<20} {relative:6.1f}x")

def main():
    """Main performance test function."""
    # Load configuration
    config = load_config()
    
    # Test path (swh-model directory)
    test_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), "swh-model")
    
    if not os.path.exists(test_path):
        print(f"Error: Test path not found: {test_path}")
        sys.exit(1)
    
    print(f"Testing performance on: {test_path}")
    print(f"Directory size: {get_directory_size(test_path):.1f} MB")
    print(f"File count: {count_files(test_path)}")
    print()
    
    # Test all implementations
    results = []
    implementations = config["implementations"]
    
    for impl_name, impl_config in implementations.items():
        if not impl_config.get("enabled", True):
            continue
            
        runner_path = os.path.join(
            os.path.dirname(__file__), 
            impl_config["runner"]
        )
        
        if not os.path.exists(runner_path):
            print(f"Warning: Runner not found: {runner_path}")
            continue
        
        result = test_implementation(impl_name, runner_path, test_path)
        results.append(result)
    
    # Print results
    print_results(results)

def get_directory_size(path: str) -> float:
    """Get directory size in MB."""
    total_size = 0
    for dirpath, dirnames, filenames in os.walk(path):
        for filename in filenames:
            filepath = os.path.join(dirpath, filename)
            if os.path.exists(filepath):
                total_size += os.path.getsize(filepath)
    return total_size / (1024 * 1024)  # Convert to MB

def count_files(path: str) -> int:
    """Count total number of files in directory."""
    count = 0
    for dirpath, dirnames, filenames in os.walk(path):
        count += len(filenames)
    return count

if __name__ == "__main__":
    main() 
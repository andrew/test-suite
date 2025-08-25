#!/usr/bin/env python3
"""
Simplified performance test for Linux source code
Tests the main implementations: Rust, Python, Git command, and Dulwich
"""

import time
import subprocess
import sys
import os
import tempfile
import shutil
from pathlib import Path

def test_rust_implementation(test_dir, num_runs=3):
    """Test Rust implementation performance"""
    print(f"Testing Rust implementation on {test_dir}...")
    times = []
    
    for i in range(num_runs):
        start_time = time.time()
        try:
            result = subprocess.run(
                ["target/release/swhid-cli", "--recursive", test_dir],
                capture_output=True, text=True, timeout=300
            )
            if result.returncode == 0:
                end_time = time.time()
                elapsed = end_time - start_time
                times.append(elapsed)
                print(f"  Run {i+1}: {elapsed:.3f}s")
            else:
                print(f"  Run {i+1}: FAILED - {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"  Run {i+1}: TIMEOUT")
        except Exception as e:
            print(f"  Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        print(f"  Average: {avg_time:.3f}s (min: {min_time:.3f}s, max: {max_time:.3f}s)")
        return avg_time, len(result.stdout.strip().split('\n'))
    else:
        print("  All runs failed")
        return None, 0

def test_python_implementation(test_dir, num_runs=3):
    """Test Python implementation performance"""
    print(f"Testing Python implementation on {test_dir}...")
    times = []
    
    for i in range(num_runs):
        start_time = time.time()
        try:
            result = subprocess.run(
                ["python", "-m", "swh.model.cli", "--type", "directory", test_dir],
                capture_output=True, text=True, timeout=300
            )
            if result.returncode == 0:
                end_time = time.time()
                elapsed = end_time - start_time
                times.append(elapsed)
                print(f"  Run {i+1}: {elapsed:.3f}s")
            else:
                print(f"  Run {i+1}: FAILED - {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"  Run {i+1}: TIMEOUT")
        except Exception as e:
            print(f"  Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        print(f"  Average: {avg_time:.3f}s (min: {min_time:.3f}s, max: {max_time:.3f}s)")
        return avg_time, len(result.stdout.strip().split('\n'))
    else:
        print("  All runs failed")
        return None, 0

def test_rust_directory_implementation(test_dir, num_runs=3):
    """Test Rust implementation performance (directory mode)"""
    print(f"Testing Rust implementation (directory mode) on {test_dir}...")
    times = []
    
    for i in range(num_runs):
        start_time = time.time()
        try:
            result = subprocess.run(
                ["target/release/swhid-cli", "--obj-type", "directory", test_dir],
                capture_output=True, text=True, timeout=300
            )
            if result.returncode == 0:
                end_time = time.time()
                elapsed = end_time - start_time
                times.append(elapsed)
                print(f"  Run {i+1}: {elapsed:.3f}s")
            else:
                print(f"  Run {i+1}: FAILED - {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"  Run {i+1}: TIMEOUT")
        except Exception as e:
            print(f"  Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        print(f"  Average: {avg_time:.3f}s (min: {min_time:.3f}s, max: {max_time:.3f}s)")
        return avg_time, len(result.stdout.strip().split('\n'))
    else:
        print("  All runs failed")
        return None, 0

def test_git_implementation(test_dir, num_runs=3):
    """Test Git command implementation performance"""
    print(f"Testing Git command implementation on {test_dir}...")
    times = []
    
    for i in range(num_runs):
        start_time = time.time()
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Initialize Git repository
                subprocess.run(["git", "init"], cwd=temp_dir, check=True)
                
                # Copy directory contents
                if os.path.isdir(test_dir):
                    for root, dirs, files in os.walk(test_dir):
                        rel_path = os.path.relpath(root, test_dir)
                        repo_dir = os.path.join(temp_dir, rel_path)
                        os.makedirs(repo_dir, exist_ok=True)
                        
                        for file in files:
                            src_file = os.path.join(root, file)
                            dst_file = os.path.join(repo_dir, file)
                            if os.path.isfile(src_file):
                                shutil.copy2(src_file, dst_file)
                
                # Add all files to Git
                subprocess.run(["git", "add", "."], cwd=temp_dir, check=True)
                
                # Get tree hash
                result = subprocess.run(
                    ["git", "write-tree"], 
                    cwd=temp_dir, capture_output=True, text=True, timeout=300
                )
                
                if result.returncode == 0:
                    end_time = time.time()
                    elapsed = end_time - start_time
                    times.append(elapsed)
                    print(f"  Run {i+1}: {elapsed:.3f}s")
                else:
                    print(f"  Run {i+1}: FAILED - {result.stderr}")
                    
        except subprocess.TimeoutExpired:
            print(f"  Run {i+1}: TIMEOUT")
        except Exception as e:
            print(f"  Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        print(f"  Average: {avg_time:.3f}s (min: {min_time:.3f}s, max: {max_time:.3f}s)")
        return avg_time, 1  # Git only returns one hash
    else:
        print("  All runs failed")
        return None, 0

def test_dulwich_implementation(test_dir, num_runs=1):
    """Test Dulwich implementation performance (much slower, so fewer runs)"""
    print(f"Testing Dulwich implementation on {test_dir}...")
    print(f"  Note: Dulwich is significantly slower, running only {num_runs} test(s)")
    times = []
    
    for i in range(num_runs):
        start_time = time.time()
        try:
            # Use the harness runner for Dulwich
            result = subprocess.run(
                ["python", "test_harness/runners/git_runner.py", test_dir, "directory"],
                capture_output=True, text=True, timeout=3600  # 1 hour timeout for Dulwich
            )
            if result.returncode == 0:
                end_time = time.time()
                elapsed = end_time - start_time
                times.append(elapsed)
                print(f"  Run {i+1}: {elapsed:.3f}s")
            else:
                print(f"  Run {i+1}: FAILED - {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"  Run {i+1}: TIMEOUT (after 1 hour)")
        except Exception as e:
            print(f"  Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        print(f"  Average: {avg_time:.3f}s (min: {min_time:.3f}s, max: {max_time:.3f}s)")
        return avg_time, 1  # Dulwich only returns one hash
    else:
        print("  All runs failed")
        return None, 0

def main():
    if len(sys.argv) != 2:
        print("Usage: python linux_performance_test.py <directory>")
        sys.exit(1)
    
    test_dir = sys.argv[1]
    if not os.path.exists(test_dir):
        print(f"Error: Directory {test_dir} does not exist")
        sys.exit(1)
    
    print("=" * 60)
    print(f"Linux Source Code Performance Test")
    print(f"Testing directory: {test_dir}")
    print("=" * 60)
    
    results = {}
    
    # Test Rust implementation
    rust_time, rust_objects = test_rust_implementation(test_dir)
    if rust_time:
        results['Rust (recursive)'] = {'time': rust_time, 'objects': rust_objects}
    
    print()
    
    # Test Rust directory implementation
    rust_dir_time, rust_dir_objects = test_rust_directory_implementation(test_dir)
    if rust_dir_time:
        results['Rust (directory)'] = {'time': rust_dir_time, 'objects': rust_dir_objects}
    
    print()
    
    # Test Python implementation
    python_time, python_objects = test_python_implementation(test_dir)
    if python_time:
        results['Python'] = {'time': python_time, 'objects': python_objects}
    
    print()
    
    # Test Git implementation
    git_time, git_objects = test_git_implementation(test_dir)
    if git_time:
        results['Git'] = {'time': git_time, 'objects': git_objects}
    
    print()
    
    # Test Dulwich implementation (much slower)
    dulwich_time, dulwich_objects = test_dulwich_implementation(test_dir)
    if dulwich_time:
        results['Dulwich'] = {'time': dulwich_time, 'objects': dulwich_objects}
    
    print()
    print("=" * 60)
    print("PERFORMANCE SUMMARY")
    print("=" * 60)
    
    if results:
        # Sort by time (fastest first)
        sorted_results = sorted(results.items(), key=lambda x: x[1]['time'])
        
        fastest_time = sorted_results[0][1]['time']
        
        print(f"{'Implementation':<15} {'Time (s)':<10} {'Objects':<10} {'Speed':<10}")
        print("-" * 50)
        
        for name, data in sorted_results:
            speed_factor = fastest_time / data['time']
            print(f"{name:<15} {data['time']:<10.3f} {data['objects']:<10} {speed_factor:<10.1f}x")
        
        print()
        print(f"Fastest: {sorted_results[0][0]} ({fastest_time:.3f}s)")
        print(f"Slowest: {sorted_results[-1][0]} ({sorted_results[-1][1]['time']:.3f}s)")
        print(f"Speed difference: {sorted_results[-1][1]['time'] / fastest_time:.1f}x")
    else:
        print("No successful runs")

if __name__ == "__main__":
    main() 
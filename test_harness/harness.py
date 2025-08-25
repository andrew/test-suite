#!/usr/bin/env python3
"""
SWHID Testing Harness for Minimal Implementation

A simplified testing harness for comparing SWHID implementations
on standardized test payloads, adapted for the minimal implementation.
"""

import argparse
import json
import os
import sys
import time
import yaml
import subprocess
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class TestResult:
    """Represents the result of a single test."""
    payload_name: str
    payload_path: str
    implementation: str
    swhid: Optional[str]
    error: Optional[str]
    duration: float
    success: bool

@dataclass
class ComparisonResult:
    """Represents the comparison of results across implementations."""
    payload_name: str
    payload_path: str
    results: Dict[str, TestResult]
    all_match: bool
    expected_swhid: Optional[str]

class SwhidHarness:
    """Simplified testing harness for SWHID implementations."""
    
    def __init__(self, config_path: str = "test_harness/config.yaml"):
        self.config_path = config_path
        self.config = self._load_config()
        self.results_dir = Path(self.config["output"]["results_dir"])
        self.results_dir.mkdir(exist_ok=True)
        
    def _load_config(self) -> Dict[str, Any]:
        """Load configuration from YAML file."""
        with open(self.config_path, 'r') as f:
            return yaml.safe_load(f)
    
    def _run_rust_test(self, payload_path: str, payload_name: str) -> TestResult:
        """Run test using our Rust implementation."""
        start_time = time.time()
        
        try:
            # Use our CLI to compute SWHID
            cmd = ["cargo", "run", "--bin", "swhid-cli", "--", payload_path]
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                cwd=os.path.dirname(os.path.dirname(__file__)),  # Go to project root
                timeout=30
            )
            
            if result.returncode != 0:
                raise RuntimeError(f"Rust implementation failed: {result.stderr}")
            
            # Parse the output
            output = result.stdout.strip()
            if not output:
                raise RuntimeError("No output from Rust implementation")
            
            # The output format is: SWHID\tfilename (optional)
            swhid = output.split('\t')[0].strip()
            
            if not swhid.startswith("swh:"):
                raise RuntimeError(f"Invalid SWHID format: {swhid}")
            
            duration = time.time() - start_time
            
            return TestResult(
                payload_name=payload_name,
                payload_path=payload_path,
                implementation="rust-minimal",
                swhid=swhid,
                error=None,
                duration=duration,
                success=True
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                payload_name=payload_name,
                payload_path=payload_path,
                implementation="rust-minimal",
                swhid=None,
                error=str(e),
                duration=duration,
                success=False
            )
    
    def _run_python_test(self, payload_path: str, payload_name: str) -> TestResult:
        """Run test using Python swh-model implementation."""
        start_time = time.time()
        
        try:
            # Use Python swh-model CLI with --no-filename for clean output
            cmd = ["python", "-m", "swh.model.cli", "--no-filename", payload_path]
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode != 0:
                raise RuntimeError(f"Python implementation failed: {result.stderr}")
            
            # Parse the output (now just the SWHID without filename)
            output = result.stdout.strip()
            if not output:
                raise RuntimeError("No output from Python implementation")
            
            # Extract SWHID from output (should be just the SWHID now)
            lines = output.split('\n')
            swhid = None
            for line in lines:
                if line.startswith("swh:"):
                    swhid = line.strip()
                    break
            
            if not swhid:
                raise RuntimeError("No SWHID found in Python output")
            
            duration = time.time() - start_time
            
            return TestResult(
                payload_name=payload_name,
                payload_path=payload_path,
                implementation="python-swh-model",
                swhid=swhid,
                error=None,
                duration=duration,
                success=True
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                payload_name=payload_name,
                payload_path=payload_path,
                implementation="python-swh-model",
                swhid=None,
                error=str(e),
                duration=duration,
                success=False
            )
    
    def _run_git_test(self, payload_path: str, payload_name: str) -> TestResult:
        """Run test using Git command line."""
        start_time = time.time()
        
        try:
            # Use git hash-object for content
            if os.path.isfile(payload_path):
                cmd = ["git", "hash-object", payload_path]
                result = subprocess.run(
                    cmd,
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                
                if result.returncode != 0:
                    raise RuntimeError(f"Git implementation failed: {result.stderr}")
                
                git_hash = result.stdout.strip()
                swhid = f"swh:1:cnt:{git_hash}"
                
            elif os.path.isdir(payload_path):
                # For directories, we need to create a tree
                # This is more complex, so we'll skip for now
                raise RuntimeError("Git directory SWHID not implemented yet")
            else:
                raise RuntimeError(f"Payload is neither file nor directory: {payload_path}")
            
            duration = time.time() - start_time
            
            return TestResult(
                payload_name=payload_name,
                payload_path=payload_path,
                implementation="git-cmd",
                swhid=swhid,
                error=None,
                duration=duration,
                success=True
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                payload_name=payload_name,
                payload_path=payload_path,
                implementation="git-cmd",
                swhid=None,
                error=str(e),
                duration=duration,
                success=False
            )
    
    def _compare_results(self, payload_name: str, payload_path: str,
                        results: Dict[str, TestResult], 
                        expected_swhid: Optional[str] = None) -> ComparisonResult:
        """Compare results across implementations."""
        # Check if all implementations succeeded
        all_success = all(r.success for r in results.values())
        
        if not all_success:
            return ComparisonResult(
                payload_name=payload_name,
                payload_path=payload_path,
                results=results,
                all_match=False,
                expected_swhid=expected_swhid
            )
        
        # Get all SWHIDs
        swhids = [r.swhid for r in results.values() if r.swhid]
        
        # Check if all SWHIDs match
        all_match = len(set(swhids)) == 1 if swhids else False
        
        # Check against expected SWHID if provided
        if expected_swhid and all_match:
            all_match = swhids[0] == expected_swhid
        
        return ComparisonResult(
            payload_name=payload_name,
            payload_path=payload_path,
            results=results,
            all_match=all_match,
            expected_swhid=expected_swhid
        )
    
    def run_tests(self, implementations: Optional[List[str]] = None,
                  categories: Optional[List[str]] = None) -> List[ComparisonResult]:
        """Run tests for specified implementations and categories."""
        if implementations is None:
            implementations = ["rust-minimal", "python-swh-model", "git-cmd"]
        
        if categories is None:
            categories = ["content", "directory"]
        
        all_results = []
        
        for category in categories:
            if category not in self.config["payloads"]:
                logger.warning(f"Category '{category}' not found in config")
                continue
                
            logger.info(f"Testing category: {category}")
            
            for payload in self.config["payloads"][category]:
                payload_path = payload["path"]
                payload_name = payload["name"]
                expected_swhid = payload.get("expected_swhid")
                
                if not os.path.exists(payload_path):
                    logger.warning(f"Payload not found: {payload_path}")
                    continue
                
                logger.info(f"Testing payload: {payload_name}")
                
                # Run tests for all implementations
                results = {}
                
                # Run Rust test
                if "rust-minimal" in implementations:
                    results["rust-minimal"] = self._run_rust_test(payload_path, payload_name)
                
                # Run Python test
                if "python-swh-model" in implementations:
                    results["python-swh-model"] = self._run_python_test(payload_path, payload_name)
                
                # Run Git test
                if "git-cmd" in implementations:
                    results["git-cmd"] = self._run_git_test(payload_path, payload_name)
                
                # Compare results
                comparison = self._compare_results(payload_name, payload_path, results, expected_swhid)
                all_results.append(comparison)
                
                # Log results
                if comparison.all_match:
                    logger.info(f"✓ {payload_name}: All implementations match")
                    for impl, result in results.items():
                        logger.info(f"  {impl}: {result.swhid} ({result.duration:.3f}s)")
                else:
                    logger.error(f"✗ {payload_name}: Implementations differ")
                    for impl, result in results.items():
                        if result.success:
                            logger.error(f"  {impl}: {result.swhid} ({result.duration:.3f}s)")
                        else:
                            logger.error(f"  {impl}: ERROR - {result.error}")
        
        return all_results
    
    def save_results(self, results: List[ComparisonResult], filename: str = None):
        """Save test results to file."""
        if filename is None:
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            filename = f"results_{timestamp}.json"
        
        filepath = self.results_dir / filename
        
        # Convert results to serializable format
        serializable_results = []
        for result in results:
            serializable_result = {
                "payload_name": result.payload_name,
                "payload_path": result.payload_path,
                "all_match": result.all_match,
                "expected_swhid": result.expected_swhid,
                "results": {}
            }
            
            for impl, test_result in result.results.items():
                serializable_result["results"][impl] = {
                    "swhid": test_result.swhid,
                    "error": test_result.error,
                    "duration": test_result.duration,
                    "success": test_result.success
                }
            
            serializable_results.append(serializable_result)
        
        # Save to file
        with open(filepath, 'w') as f:
            json.dump(serializable_results, f, indent=2)
        
        logger.info(f"Results saved to: {filepath}")
        return filepath
    
    def print_summary(self, results: List[ComparisonResult]):
        """Print a summary of test results."""
        total_tests = len(results)
        successful_tests = sum(1 for r in results if r.all_match)
        failed_tests = total_tests - successful_tests
        
        print(f"\n{'='*60}")
        print(f"TEST SUMMARY")
        print(f"{'='*60}")
        print(f"Total tests: {total_tests}")
        print(f"Successful: {successful_tests}")
        print(f"Failed: {failed_tests}")
        print(f"Success rate: {(successful_tests/total_tests)*100:.1f}%")
        
        if failed_tests > 0:
            print(f"\nFailed tests:")
            for result in results:
                if not result.all_match:
                    print(f"  - {result.payload_name}: {result.payload_path}")
                    for impl, test_result in result.results.items():
                        if test_result.success:
                            print(f"    {impl}: {test_result.swhid}")
                        else:
                            print(f"    {impl}: ERROR - {test_result.error}")
        
        print(f"{'='*60}")

def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="SWHID Testing Harness")
    parser.add_argument("--impl", nargs="+", help="Implementations to test")
    parser.add_argument("--category", nargs="+", help="Test categories to run")
    parser.add_argument("--output", help="Output filename for results")
    parser.add_argument("--config", default="test_harness/config.yaml", help="Config file path")
    
    args = parser.parse_args()
    
    # Initialize harness
    harness = SwhidHarness(args.config)
    
    # Run tests
    results = harness.run_tests(args.impl, args.category)
    
    # Save results
    if args.output:
        harness.save_results(results, args.output)
    else:
        harness.save_results(results)
    
    # Print summary
    harness.print_summary(results)
    
    # Exit with error code if any tests failed
    if any(not r.all_match for r in results):
        sys.exit(1)

if __name__ == "__main__":
    main() 
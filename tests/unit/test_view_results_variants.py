#!/usr/bin/env python3
"""Unit tests for variant registry system in view_results.py"""

import unittest
from scripts.view_results import VariantRegistry


class TestVariantRegistry(unittest.TestCase):
    """Test VariantRegistry class."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.registry = VariantRegistry()
    
    def test_default_variants_registered(self):
        """Test that default v1 and v2 variants are registered."""
        variants = self.registry.list_variants()
        self.assertIn('v1_sha1_hex', variants)
        self.assertIn('v2_sha256_hex', variants)
    
    def test_v1_sha1_hex_detection(self):
        """Test detection of v1 SHA1 hex SWHID."""
        swhid = 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
        variant = self.registry.get_variant_for_swhid(swhid)
        self.assertEqual(variant, 'v1_sha1_hex')
        
        expected_key = self.registry.get_expected_key(variant)
        self.assertEqual(expected_key, 'swhid')
    
    def test_v2_sha256_hex_detection(self):
        """Test detection of v2 SHA256 hex SWHID."""
        swhid = 'swh:2:cnt:473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813'
        variant = self.registry.get_variant_for_swhid(swhid)
        self.assertEqual(variant, 'v2_sha256_hex')
        
        expected_key = self.registry.get_expected_key(variant)
        self.assertEqual(expected_key, 'expected_swhid_sha256')
    
    def test_invalid_swhid_returns_none(self):
        """Test that invalid SWHID strings return None."""
        invalid_cases = [
            '',
            'invalid',
            'swh:',
            'swh:1:',
            'swh:1:cnt',
        ]
        for invalid in invalid_cases:
            variant = self.registry.get_variant_for_swhid(invalid)
            self.assertIsNone(variant, f"Expected None for '{invalid}'")
    
    def test_register_new_variant(self):
        """Test registering a new variant."""
        self.registry.register_variant('v2_sha256_base64', {
            'version': 2,
            'hash_algo': 'sha256',
            'serialization': 'base64',
            'expected_key': 'expected_swhid_sha256_base64',
            'swhid_prefix': 'swh:2:',
            'hash_length': 44,
        })
        
        self.assertIn('v2_sha256_base64', self.registry.list_variants())
        config = self.registry.get_variant_config('v2_sha256_base64')
        self.assertEqual(config['hash_length'], 44)
        self.assertEqual(config['expected_key'], 'expected_swhid_sha256_base64')
    
    def test_register_variant_missing_key_raises(self):
        """Test that registering variant with missing required key raises ValueError."""
        with self.assertRaises(ValueError):
            self.registry.register_variant('invalid', {
                'version': 1,
                'hash_algo': 'sha1',
                # Missing other required keys
            })
    
    def test_get_variant_config(self):
        """Test getting variant configuration."""
        config = self.registry.get_variant_config('v1_sha1_hex')
        self.assertIsNotNone(config)
        self.assertEqual(config['version'], 1)
        self.assertEqual(config['hash_algo'], 'sha1')
        self.assertEqual(config['serialization'], 'hex')
        self.assertEqual(config['hash_length'], 40)
    
    def test_get_variant_config_nonexistent(self):
        """Test getting config for non-existent variant returns None."""
        config = self.registry.get_variant_config('nonexistent')
        self.assertIsNone(config)
    
    def test_serialization_detection_hex(self):
        """Test hex serialization format detection."""
        hex_hash = 'e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
        serialization = self.registry._detect_serialization_format(hex_hash)
        self.assertEqual(serialization, 'hex')
    
    def test_serialization_detection_base64(self):
        """Test base64 serialization format detection."""
        base64_hash = '5p3im7Ld1kQ4uymud12tws5M1Tk='
        serialization = self.registry._detect_serialization_format(base64_hash)
        self.assertEqual(serialization, 'base64')
    
    def test_hash_algo_detection_from_length(self):
        """Test hash algorithm detection from length."""
        self.assertEqual(self.registry._detect_hash_algo_from_length(40), 'sha1')
        self.assertEqual(self.registry._detect_hash_algo_from_length(64), 'sha256')
        self.assertEqual(self.registry._detect_hash_algo_from_length(128), 'sha512')
        self.assertEqual(self.registry._detect_hash_algo_from_length(44), 'sha256')  # base64
        self.assertEqual(self.registry._detect_hash_algo_from_length(99), 'unknown')  # unknown


if __name__ == '__main__':
    unittest.main()


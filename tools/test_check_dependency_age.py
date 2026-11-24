"""Offline tests for check_dependency_age's pure parts.

Run from the repository root:
    python3 -I -m unittest discover -s tools -p 'test_*.py'
"""

import json
import unittest
from datetime import datetime, timezone

import check_dependency_age as check


class CrateIndexUrl(unittest.TestCase):
    def test_follows_the_sparse_index_layout(self):
        self.assertEqual(check.crate_index_url("a"), "https://index.crates.io/1/a")
        self.assertEqual(check.crate_index_url("cc"), "https://index.crates.io/2/cc")
        self.assertEqual(check.crate_index_url("url"), "https://index.crates.io/3/u/url")
        self.assertEqual(check.crate_index_url("serde"), "https://index.crates.io/se/rd/serde")

    def test_lowercases_names(self):
        self.assertEqual(check.crate_index_url("Serde"), "https://index.crates.io/se/rd/serde")


class CargoPackages(unittest.TestCase):
    LOCK = f"""
version = 4
[[package]]
name = "domain"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.0"
source = "{check.CRATES_IO}"

[[package]]
name = "forked"
version = "0.2.0"
source = "git+https://example.invalid/forked#abc123"
"""

    def test_splits_registry_from_unverifiable_and_skips_paths(self):
        registry, other = check.cargo_packages(self.LOCK)
        self.assertEqual(registry, [("serde", "1.0.0")])
        self.assertEqual(other, [("forked", "0.2.0")])


class NpmPackages(unittest.TestCase):
    LOCK = json.dumps(
        {
            "lockfileVersion": 3,
            "packages": {
                "": {"name": "root"},
                "node_modules/@scope/pkg": {
                    "version": "1.2.3",
                    "resolved": "https://registry.npmjs.org/@scope/pkg/-/pkg-1.2.3.tgz",
                },
                "node_modules/from-git": {
                    "version": "0.1.0",
                    "resolved": "git+ssh://git@example.invalid/x.git#abc",
                },
                "node_modules/linked": {"link": True, "resolved": "packages/linked"},
            },
        }
    )

    def test_splits_registry_from_unverifiable_and_skips_root_and_links(self):
        registry, other = check.npm_packages(self.LOCK)
        self.assertEqual(registry, [("@scope/pkg", "1.2.3")])
        self.assertEqual(other, [("from-git", "0.1.0")])


class ParseAllow(unittest.TestCase):
    def test_strips_comments_and_blank_lines(self):
        text = "# header\n\ncrates:url@2.5.9   # RUSTSEC fix\nnpm:x@1.0.0\n"
        self.assertEqual(check.parse_allow(text), {"crates:url@2.5.9", "npm:x@1.0.0"})


class IsTooNew(unittest.TestCase):
    NOW = datetime(2026, 11, 24, 12, 0, tzinfo=timezone.utc)

    def test_the_boundary_is_exactly_min_days(self):
        self.assertFalse(check.is_too_new("2026-11-10T12:00:00Z", self.NOW))
        self.assertTrue(check.is_too_new("2026-11-10T12:00:01Z", self.NOW))

    def test_accepts_offsets_and_fractions(self):
        self.assertFalse(check.is_too_new("2026-10-01T00:00:00.123+02:00", self.NOW))


if __name__ == "__main__":
    unittest.main()

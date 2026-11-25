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

[[package]]
name = "mirrored"
version = "1.0.0"
source = "sparse+https://example.invalid/index/"
"""

    def test_splits_registry_from_unverifiable_and_skips_paths(self):
        registry, other = check.cargo_packages(self.LOCK)
        self.assertEqual(registry, [("serde", "1.0.0")])
        self.assertEqual(other, [("forked", "0.2.0"), ("mirrored", "1.0.0")])


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
                "node_modules/parent/node_modules/bundled": {"version": "2.0.0", "inBundle": True},
                "node_modules/own-bundle": {
                    "version": "3.0.0",
                    "inBundle": True,
                    "resolved": "https://registry.npmjs.org/own-bundle/-/own-bundle-3.0.0.tgz",
                },
                "node_modules/own-bundle/node_modules/own-dep": {
                    "version": "4.0.0",
                    "inBundle": True,
                    "resolved": "https://registry.npmjs.org/own-dep/-/own-dep-4.0.0.tgz",
                },
                "node_modules/other-registry": {
                    "version": "1.0.0",
                    "resolved": "https://npm.example.invalid/other-registry/-/other-registry-1.0.0.tgz",
                },
                "node_modules/lodash": {
                    "version": "4.17.21",
                    "resolved": "https://registry.npmjs.org/evilpkg/-/evilpkg-9.9.9.tgz",
                },
            },
        }
    )

    def test_splits_registry_from_unverifiable_and_skips_root_links_and_bundles(self):
        registry, other = check.npm_packages(self.LOCK)
        # The project's own bundled dependencies (and theirs) are still
        # downloaded, so checked; only tarball-shipped entries are skipped.
        self.assertEqual(
            registry, [("@scope/pkg", "1.2.3"), ("own-bundle", "3.0.0"), ("own-dep", "4.0.0")]
        )
        # A tarball for another package or version can't vouch for this entry.
        self.assertEqual(
            other, [("from-git", "0.1.0"), ("lodash", "4.17.21"), ("other-registry", "1.0.0")]
        )


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


class Evaluate(unittest.TestCase):
    NOW = datetime(2026, 11, 24, tzinfo=timezone.utc)
    OLD, NEW = "2026-10-01T00:00:00Z", "2026-11-20T00:00:00Z"

    def run_(self, published, allowed=(), unverifiable=()):
        return check.evaluate(published, set(allowed), list(unverifiable), self.NOW)

    def test_old_packages_pass(self):
        self.assertEqual(self.run_({"crates:a@1": self.OLD}), ([], []))

    def test_new_or_undated_packages_fail(self):
        failures, _ = self.run_({"crates:a@1": self.NEW, "npm:b@1": None})
        self.assertEqual(len(failures), 2)

    def test_an_allowed_new_package_passes_and_is_not_stale(self):
        self.assertEqual(self.run_({"crates:a@1": self.NEW}, allowed=["crates:a@1"]), ([], []))

    def test_allow_entries_that_are_old_or_unlocked_are_stale(self):
        failures, stale = self.run_(
            {"crates:a@1": self.OLD}, allowed=["crates:a@1", "npm:gone@1"]
        )
        self.assertEqual((failures, stale), ([], ["crates:a@1", "npm:gone@1"]))

    def test_an_allow_entry_covers_only_its_exact_version(self):
        failures, _ = self.run_({"npm:a@2": self.NEW}, allowed=["npm:a@1"])
        self.assertEqual(len(failures), 1)

    def test_unverifiable_sources_fail_even_if_allowed(self):
        failures, _ = self.run_({}, allowed=["crates:x@1"], unverifiable=["crates:x@1"])
        self.assertEqual(len(failures), 1)


if __name__ == "__main__":
    unittest.main()

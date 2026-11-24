"""Fail if any locked dependency was published less than MIN_DAYS ago.

Fresh releases are where compromised publishes usually get caught, so the
project only adopts versions that have been public for a while. Checks
Cargo.lock (crates.io) and web/package-lock.json (npm registry). A
dependency from anywhere else (git, a tarball URL) has no publish time to
check, so it fails too; local path and workspace packages are skipped.

Exceptions, each with a reason, go in .dependency-age-allow:
    crates:name@version  # why
    npm:name@version     # why
Entries that are no longer locked, or have since become old enough, are
reported so the list stays short.

Usage, from the repository root:  python3 -I tools/check_dependency_age.py
Standard library only, so it runs before any dependency is installed.
"""

import json
import sys
import time
import tomllib
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timedelta, timezone
from pathlib import Path

MIN_DAYS = 14
ROOT = Path(__file__).resolve().parents[1]
USER_AGENT = "ricochet-robots-dependency-age-check"
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
NPM_REGISTRY = "https://registry.npmjs.org/"


def fetch(url, attempts=2):
    """GET `url`, retrying once: one flaky request shouldn't fail CI."""
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return response.read().decode()
        except OSError:
            if attempt + 1 == attempts:
                raise
            time.sleep(2)
    raise AssertionError("unreachable")


def crate_index_url(name):
    """The crates.io sparse-index file for `name` (see the index layout)."""
    name = name.lower()
    if len(name) <= 2:
        return f"https://index.crates.io/{len(name)}/{name}"
    if len(name) == 3:
        return f"https://index.crates.io/3/{name[0]}/{name}"
    return f"https://index.crates.io/{name[:2]}/{name[2:4]}/{name}"


def crate_published(name, version):
    for line in fetch(crate_index_url(name)).splitlines():
        entry = json.loads(line)
        if entry["vers"] == version:
            return entry.get("pubtime")
    return None


def npm_published(name, version):
    metadata = json.loads(fetch(f"{NPM_REGISTRY}{name.replace('/', '%2F')}"))
    return metadata.get("time", {}).get(version)


def cargo_packages(lock_text):
    """(registry packages, unverifiable sources) in a Cargo.lock."""
    registry, other = set(), set()
    for package in tomllib.loads(lock_text).get("package", []):
        source = package.get("source")
        if source is None:
            continue  # a path dependency, i.e. our own crates
        key = (package["name"], package["version"])
        (registry if source == CRATES_IO else other).add(key)
    return sorted(registry), sorted(other)


def npm_packages(lock_text):
    """(registry packages, unverifiable sources) in a package-lock.json v3."""
    registry, other = set(), set()
    for path, info in json.loads(lock_text).get("packages", {}).items():
        if not path or info.get("link"):
            continue  # the root package or a workspace link
        name = info.get("name") or path.rsplit("node_modules/", 1)[-1]
        key = (name, info.get("version", "?"))
        resolved = info.get("resolved", "")
        (registry if resolved.startswith(NPM_REGISTRY) else other).add(key)
    return sorted(registry), sorted(other)


def parse_allow(text):
    """Entries of an allow file, without comments or blank lines."""
    entries = set()
    for line in text.splitlines():
        entry = line.split("#", 1)[0].strip()
        if entry:
            entries.add(entry)
    return entries


def is_too_new(published, now, min_days=MIN_DAYS):
    """Whether an ISO 8601 publish time is less than `min_days` before `now`."""
    when = datetime.fromisoformat(published.replace("Z", "+00:00"))
    return when > now - timedelta(days=min_days)


def main():
    now = datetime.now(timezone.utc)
    crates, crates_other = cargo_packages((ROOT / "Cargo.lock").read_text())
    npm, npm_other = npm_packages((ROOT / "web" / "package-lock.json").read_text())
    jobs = [("crates", n, v, crate_published) for n, v in crates]
    jobs += [("npm", n, v, npm_published) for n, v in npm]
    with ThreadPoolExecutor(max_workers=16) as pool:
        times = list(pool.map(lambda job: job[3](job[1], job[2]), jobs))

    allow_path = ROOT / ".dependency-age-allow"
    allowed = parse_allow(allow_path.read_text()) if allow_path.exists() else set()
    failures, stale = [], set(allowed)
    for (ecosystem, name, version, _), published in zip(jobs, times):
        key = f"{ecosystem}:{name}@{version}"
        new = published is None or is_too_new(published, now)
        if key in allowed:
            if new:
                stale.discard(key)  # still needed
        elif published is None:
            failures.append(f"no publish time found: {key}")
        elif new:
            failures.append(f"too new: {key} (published {published[:10]})")
    for ecosystem, packages in (("crates", crates_other), ("npm", npm_other)):
        failures += [f"not from the registry, can't check its age: {ecosystem}:{n}@{v}" for n, v in packages]

    print(f"Checked {len(jobs)} packages against a {MIN_DAYS}-day minimum age.")
    for line in failures:
        print(f"  {line}")
    for key in sorted(stale):
        print(f"  note: allow-list entry no longer needed: {key}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())

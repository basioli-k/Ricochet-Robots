"""Fail if any locked dependency was published less than MIN_DAYS ago.

Fresh releases are where compromised publishes usually get caught, so the
project only adopts versions that have been public for a while. Checks
Cargo.lock (crates.io) and web/package-lock.json (npm registry).

Exceptions, each with a reason, go in .dependency-age-allow:
    crates:name@version  # why
    npm:name@version     # why

Usage, from the repository root:  python3 -I tools/check_dependency_age.py
Standard library only, so it runs before any dependency is installed.
"""

import json
import sys
import tomllib
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timedelta, timezone
from pathlib import Path

MIN_DAYS = 14
ROOT = Path(__file__).resolve().parents[1]
USER_AGENT = "ricochet-robots-dependency-age-check"


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read().decode()


def crate_index_url(name):
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
    metadata = json.loads(fetch(f"https://registry.npmjs.org/{name.replace('/', '%2F')}"))
    return metadata.get("time", {}).get(version)


def locked_crates():
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    return sorted(
        {(p["name"], p["version"]) for p in lock.get("package", []) if "crates.io-index" in p.get("source", "")}
    )


def locked_npm_packages():
    lock = json.loads((ROOT / "web" / "package-lock.json").read_text())
    packages = set()
    for path, info in lock.get("packages", {}).items():
        if path and "registry.npmjs.org" in info.get("resolved", ""):
            name = info.get("name") or path.rsplit("node_modules/", 1)[-1]
            packages.add((name, info["version"]))
    return sorted(packages)


def allowed():
    path = ROOT / ".dependency-age-allow"
    if not path.exists():
        return set()
    entries = set()
    for line in path.read_text().splitlines():
        entry = line.split("#", 1)[0].strip()
        if entry:
            entries.add(entry)
    return entries


def main():
    cutoff = datetime.now(timezone.utc) - timedelta(days=MIN_DAYS)
    jobs = [("crates", n, v, crate_published) for n, v in locked_crates()]
    jobs += [("npm", n, v, npm_published) for n, v in locked_npm_packages()]
    with ThreadPoolExecutor(max_workers=16) as pool:
        times = list(pool.map(lambda job: job[3](job[1], job[2]), jobs))

    exceptions = allowed()
    too_new, unknown = [], []
    for (ecosystem, name, version, _), published in zip(jobs, times):
        key = f"{ecosystem}:{name}@{version}"
        if key in exceptions:
            continue
        if published is None:
            unknown.append(key)
        elif datetime.fromisoformat(published.replace("Z", "+00:00")) > cutoff:
            too_new.append(f"{key} (published {published[:10]})")

    print(f"Checked {len(jobs)} packages against a {MIN_DAYS}-day minimum age.")
    for line in too_new:
        print(f"  too new: {line}")
    for line in unknown:
        print(f"  no publish time found: {line}")
    return 1 if too_new or unknown else 0


if __name__ == "__main__":
    sys.exit(main())

"""This module generates releases.rs for the huak_python_manager crate."""
import re
import subprocess
from typing import NamedTuple
import requests
from pathlib import Path
from urllib.parse import unquote
import polars as pl


FILE = Path(__file__)
ROOT = Path(
    subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
)
CRATE = "huak_python_manager"
TOKEN = (FILE.parent / ".github_token").read_text().strip()

RELEASE_URL = "https://api.github.com/repos/indygreg/python-build-standalone/releases"
HEADERS = headers = {
    "Accept": "application/vnd.github+json",
    "Authorization": f"Bearer {TOKEN}",
}

VERSION_PATTERN = re.compile(r"cpython-(\d+\.\d+\.\d+)")
OS_PATTERN = re.compile(r"-(windows|apple|linux)-")
ARCHITECTURE_PATTERN = re.compile(r"-(aarch64|i686|x86_64|x86)-")
BUILD_PATTERN = re.compile(r"-(pgo\+lto|pgo)-")


class Release(NamedTuple):
    kind: str
    version: str
    os: str
    architecture: str
    build_configuration: str
    checksum: str
    url: str

    def to_rust_string(self) -> str:
        (major, minor, patch) = self.version.split(".")
        version = f"Version::new({major}, {minor}, {patch})"
        return f"""\
Release::new("{self.kind}", {version}, "{self.os}", "{self.architecture}", "{self.build_configuration}", "{self.checksum}", "{self.url}")\
"""  # noqa


session = requests.Session()
release_json = session.get(RELEASE_URL).json()


def is_checksum_url(url: str) -> bool:
    return url.endswith(".sha256") or url.endswith("SHA256SUMS")


def get_checksum(url: str) -> str | None:
    res = session.get(url)
    res.raise_for_status()
    return res.text.strip()


path = FILE.parent / "generated_python_releases.parquet"
generated = (
    pl.DataFrame({"url": [], "string": []}, schema={"url": pl.Utf8, "string": pl.Utf8})
    if not path.exists()
    else pl.read_parquet(path)
)
new_releases = {"url": [], "string": []}

# Identify releases with checksums published.
has_checksum = set()
for release in release_json:
    for asset in release["assets"]:
        if asset["browser_download_url"].endswith(".sha256"):
            has_checksum.add(asset["browser_download_url"].removesuffix(".sha256"))


module = f"""\
//! This file was generated with `{FILE.name}`.

#[allow(dead_code)]
#[rustfmt::skip]
pub const RELEASES: &[Release] = &[\
"""  # noqa
for release in release_json:
    for asset in release["assets"]:
        # Avoid making requests for releases we've already generated.
        matching = generated.filter(pl.col("url").eq(asset["browser_download_url"]))
        if not matching.is_empty():
            string = matching.select(pl.col("string")).to_series()[0]
            module += "\n\t" + string + ","
            continue

        # Skip any releases that don't have checksums
        if asset["browser_download_url"] not in has_checksum:
            print(f"no checksum for {asset['name']}")
            continue

        url = unquote(asset["browser_download_url"])

        # Skip builds not included in the pattern
        build_matches = re.search(BUILD_PATTERN, url)
        if not build_matches:
            continue
        build_str = build_matches.group(1)

        # Skip architectures not included in the pattern
        arch_matches = re.search(ARCHITECTURE_PATTERN, url)
        if not arch_matches:
            continue
        arch_str = arch_matches.group(1)

        checksum_str = get_checksum(asset["browser_download_url"] + ".sha256")
        version_str = re.search(VERSION_PATTERN, url).group(1)
        os_str = re.search(OS_PATTERN, url).group(1)
        release = Release(
            "cpython",
            version_str,
            os_str,
            arch_str,
            build_str,
            checksum_str,
            asset["browser_download_url"],
        )
        new_releases["url"].append(asset["browser_download_url"])
        new_releases["string"].append(release.to_rust_string())
        module += "\n\t" + release.to_rust_string() + ","
module += """\n];

#[derive(Copy, Clone)]
pub struct Release<'a> {
    pub kind: &'a str,
    pub version: Version,
    pub os: &'a str,
    pub architecture: &'a str,
    pub build_configuration: &'a str,
    pub checksum: &'a str,
    pub url: &'a str,
}

impl Release<'static> {
    #[allow(dead_code)]
    const fn new(
        kind: &'static str,
        version: Version,
        os: &'static str,
        architecture: &'static str,
        build_configuration: &'static str,
        checksum: &'static str,
        url: &'static str,
    ) -> Self {
        Self {
            kind,
            version,
            os,
            architecture,
            build_configuration,
            checksum,
            url,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    #[allow(dead_code)]
    const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}
"""

path = ROOT / "crates" / CRATE / "src" / "releases.rs"
path.write_text(module)

new_releases = pl.DataFrame(new_releases, schema={"url": pl.Utf8, "string": pl.Utf8})
path = FILE.parent / "generated_python_releases.parquet"
pl.concat((generated, new_releases)).write_parquet(path)

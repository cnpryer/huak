use serde_derive::{Deserialize, Serialize};

// TODO: Support more than https://pypi.org/pypi/<package name>/json
//       Ex: See https://peps.python.org/pep-0503/
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageIndexData {
    pub info: PackageInfo,
    last_serial: u64,
    releases: serde_json::value::Value,
    urls: Vec<serde_json::value::Value>,
    vulnerabilities: Vec<serde_json::value::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageInfo {
    pub author: String,
    pub author_email: String,
    pub bugtrack_url: serde_json::value::Value,
    pub classifiers: Vec<String>,
    pub description: String,
    pub description_content_type: String,
    pub docs_url: serde_json::value::Value,
    pub download_url: serde_json::value::Value,
    pub downloads: serde_json::value::Value,
    pub home_page: serde_json::value::Value,
    pub keywords: serde_json::value::Value,
    pub license: serde_json::value::Value,
    pub maintainer: serde_json::value::Value,
    pub maintainer_email: serde_json::value::Value,
    pub name: String,
    pub package_url: String,
    pub platform: serde_json::value::Value,
    pub project_url: String,
    pub project_urls: serde_json::value::Value,
    pub release_url: String,
    pub requires_dist: serde_json::value::Value,
    pub requires_python: String,
    pub summary: String,
    pub version: String,
    pub yanked: bool,
    pub yanked_reason: serde_json::value::Value,
}

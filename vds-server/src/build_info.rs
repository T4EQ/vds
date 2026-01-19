/// VDS build information compiled into the vds-server binary
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct BuildInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub git_hash: Option<String>,
    pub authors: &'static str,
    pub homepage: &'static str,
    pub license: &'static str,
    pub repository: &'static str,
    pub profile: &'static str,
    pub rustc_version: &'static str,
    pub features: &'static str,
}

std::include!(std::concat!(std::env!("OUT_DIR"), "/built.rs"));

#[cfg(feature = "git2")]
fn git_hash() -> Option<String> {
    GIT_COMMIT_HASH.map(|git_hash| {
        let dirty = if GIT_DIRTY.is_some_and(|v| v) {
            "-dirty"
        } else {
            ""
        };
        format!("{git_hash}{dirty}")
    })
}

#[cfg(not(feature = "git2"))]
fn git_hash() -> Option<String> {
    std::option_env!("VDS_SERVER_NIX_GIT_REVISION").map(|git_hash| git_hash.to_string())
}

pub fn get() -> BuildInfo {
    BuildInfo {
        name: PKG_NAME,
        version: PKG_VERSION,
        git_hash: git_hash(),
        authors: PKG_AUTHORS,
        homepage: PKG_HOMEPAGE,
        license: PKG_LICENSE,
        repository: PKG_REPOSITORY,
        profile: PROFILE,
        rustc_version: RUSTC_VERSION,
        features: FEATURES_STR,
    }
}

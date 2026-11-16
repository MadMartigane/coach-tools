//! The application version, fixed at compile time from the workspace manifest. Unrelated to the storage schema version in platform::storage.

/// The application version. Its source is `[workspace.package] version` in the
/// root `Cargo.toml`, inherited by this crate through `version.workspace = true`.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The short commit SHA of a pre-production build. `scripts/deploy.sh` sets
/// `COACH_TOOLS_BUILD_SHA` for pre-production builds only; production builds leave it unset.
const BUILD_SHA: Option<&str> = option_env!("COACH_TOOLS_BUILD_SHA");

/// The version shown on the home screen: `vX.Y.Z`, or `vX.Y.Z+<sha>` when a build SHA is set.
pub fn version_label() -> String {
    label(APP_VERSION, BUILD_SHA)
}

/// Formats `version` with `build_sha` appended as SemVer build metadata when it is non-empty.
fn label(version: &str, build_sha: Option<&str>) -> String {
    match build_sha.filter(|sha| !sha.is_empty()) {
        Some(sha) => format!("v{version}+{sha}"),
        None => format!("v{version}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_without_build_sha_is_the_plain_version() {
        assert_eq!(label("0.1.1", None), "v0.1.1");
    }

    #[test]
    fn label_with_build_sha_appends_it_as_build_metadata() {
        assert_eq!(label("0.1.1", Some("abc1234")), "v0.1.1+abc1234");
    }

    #[test]
    fn empty_build_sha_is_ignored() {
        assert_eq!(label("0.1.1", Some("")), "v0.1.1");
    }

    #[test]
    fn app_version_matches_the_workspace_manifest() {
        let manifest = include_str!("../../Cargo.toml");
        let manifest_versions: Vec<&str> = manifest
            .lines()
            .filter_map(|line| line.strip_prefix("version = \""))
            .filter_map(|rest| rest.strip_suffix('"'))
            .collect();
        assert_eq!(manifest_versions, vec![APP_VERSION]);
    }
}

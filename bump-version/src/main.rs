//! Prints the next SemVer version of the workspace; used by scripts/deploy.sh prod.

use std::process::ExitCode;
use std::str::FromStr;

const USAGE: &str = "usage: bump-version <MAJOR.MINOR.PATCH> <patch|minor|major>";

/// The version component that a release increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    /// Increments the patch number: `0.1.0` becomes `0.1.1`.
    Patch,
    /// Increments the minor number and resets the patch: `0.1.9` becomes `0.2.0`.
    Minor,
    /// Increments the major number and resets minor and patch: `1.4.2` becomes `2.0.0`.
    Major,
}

impl FromStr for Level {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "patch" => Ok(Self::Patch),
            "minor" => Ok(Self::Minor),
            "major" => Ok(Self::Major),
            _ => Err(format!(
                "unknown bump level '{text}'; expected patch, minor or major"
            )),
        }
    }
}

/// Parses one canonical version number: ASCII digits, no leading zero, fits in `u64`.
fn parse_component(component: &str) -> Option<u64> {
    let is_canonical = !component.is_empty()
        && component.bytes().all(|byte| byte.is_ascii_digit())
        && (component == "0" || !component.starts_with('0'));
    if is_canonical {
        component.parse().ok()
    } else {
        None
    }
}

/// Splits a strict `MAJOR.MINOR.PATCH` version into its three numbers.
fn parse_version(current: &str) -> Option<(u64, u64, u64)> {
    let mut components = current.split('.');
    let (Some(major), Some(minor), Some(patch), None) = (
        components.next(),
        components.next(),
        components.next(),
        components.next(),
    ) else {
        return None;
    };
    Some((
        parse_component(major)?,
        parse_component(minor)?,
        parse_component(patch)?,
    ))
}

fn increment(number: u64, current: &str) -> Result<u64, String> {
    number
        .checked_add(1)
        .ok_or_else(|| format!("version '{current}' cannot be bumped: number overflow"))
}

fn next_version(current: &str, level: Level) -> Result<String, String> {
    let (major, minor, patch) = parse_version(current)
        .ok_or_else(|| format!("malformed version '{current}'; expected MAJOR.MINOR.PATCH"))?;
    let (next_major, next_minor, next_patch) = match level {
        Level::Patch => (major, minor, increment(patch, current)?),
        Level::Minor => (major, increment(minor, current)?, 0),
        Level::Major => (increment(major, current)?, 0, 0),
    };
    Ok(format!("{next_major}.{next_minor}.{next_patch}"))
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [current, level_text] = arguments.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    match level_text
        .parse::<Level>()
        .and_then(|level| next_version(current, level))
    {
        Ok(next) => {
            println!("{next}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_increments_patch() {
        assert_eq!(next_version("0.1.0", Level::Patch), Ok("0.1.1".to_string()));
    }

    #[test]
    fn minor_increments_minor_and_resets_patch() {
        assert_eq!(next_version("0.1.9", Level::Minor), Ok("0.2.0".to_string()));
    }

    #[test]
    fn major_increments_major_and_resets_minor_and_patch() {
        assert_eq!(next_version("1.4.2", Level::Major), Ok("2.0.0".to_string()));
    }

    #[test]
    fn malformed_versions_are_rejected() {
        let malformed_versions = [
            "",
            "1.2",
            "1.2.3.4",
            "1.a.3",
            "1..3",
            "01.2.3",
            "1.2.3-rc.1",
            " 1.2.3",
        ];
        for version in malformed_versions {
            let result = next_version(version, Level::Patch);
            assert!(
                result
                    .as_ref()
                    .is_err_and(|message| message.starts_with("malformed version")),
                "expected '{version}' to be rejected, got {result:?}"
            );
        }
    }

    #[test]
    fn overflow_is_rejected() {
        let version = format!("0.1.{}", u64::MAX);
        assert!(next_version(&version, Level::Patch).is_err());
    }

    #[test]
    fn levels_parse_exactly() {
        assert_eq!("patch".parse::<Level>(), Ok(Level::Patch));
        assert_eq!("minor".parse::<Level>(), Ok(Level::Minor));
        assert_eq!("major".parse::<Level>(), Ok(Level::Major));
        assert!("Patch".parse::<Level>().is_err());
        assert!("".parse::<Level>().is_err());
    }
}

use anyhow::Context as _;
use anyhow::Result;

/// Parse kernel version string in the format "major.minor.patch".
///
/// # Arguments
/// * `s` - Raw input string, expected to be in format "X.Y.Z" (e.g., "5.4.0")
///
/// # Returns
/// * `Ok((u8, u8, u8))` - The parsed kernel version as a tuple of (major, minor, patch)
/// * `Err(anyhow::Error)` - If the format is invalid
///
/// # Examples
/// ```
/// use bpflint::parse_kernel_version;
/// assert_eq!(parse_kernel_version("5.4.0").unwrap(), (5, 4, 0));
/// assert!(parse_kernel_version("5.4").is_err());
/// assert!(parse_kernel_version("5.a.0").is_err());
/// ```
pub fn parse_kernel_version(s: &str) -> Result<(u8, u8, u8)> {
    let parts: Vec<&str> = s.split('.').collect();
    
    if parts.len() != 3 {
        anyhow::bail!(
            "kernel version must be in format 'major.minor.patch' (e.g., '5.4.0'), got '{}'",
            s
        );
    }
    
    let major = parts[0].parse::<u8>()
        .with_context(|| format!("invalid major version: '{}' (must be a non-negative integer)", parts[0]))?;
    
    let minor = parts[1].parse::<u8>()
        .with_context(|| format!("invalid minor version: '{}' (must be a non-negative integer)", parts[1]))?;
    
    let patch = parts[2].parse::<u8>()
        .with_context(|| format!("invalid patch version: '{}' (must be a non-negative integer)", parts[2]))?;
    
    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kernel_version_valid() {
        let (major, minor, patch) = parse_kernel_version("5.4.0").unwrap();
        assert_eq!((major, minor, patch), (5, 4, 0));

        let (major, minor, patch) = parse_kernel_version("84.71.23").unwrap();
        assert_eq!((major, minor, patch), (84, 71, 23));
    }

    #[test]
    fn test_parse_kernel_version_invalid_parts() {
        let result = parse_kernel_version("5.bfp.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_kernel_version_too_many_parts() {
        let result = parse_kernel_version("5.1.0.9");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_kernel_version_too_few_parts() {
        let result = parse_kernel_version("4.8");
        assert!(result.is_err());
    }
}

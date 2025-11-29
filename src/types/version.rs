use anyhow::{Result, bail};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Version {
    pub fn latest() -> Self {
        Version { major: 1, minor: 0 }
    }

    pub fn fail_if_not_supported(&self) -> Result<()> {
        if self.major != 1 {
            bail!(
                "Version {} is not supported. Only version 1 is supported",
                self
            );
        }
        Ok(())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            bail!("Got invalid version format: {}. Expected format: 1.0", s);
        }

        Ok(Version {
            major: parts[0].parse()?,
            minor: parts[1].parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let mut version: Version = "1.2".parse().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);

        version = "25.63".parse().unwrap();
        assert_eq!(version.major, 25);
        assert_eq!(version.minor, 63);
    }
}

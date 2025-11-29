use anyhow::{Result, bail};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl FromStr for TimeSignature {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            bail!("Invalid time signature format: {}", s);
        }

        let numerator = parts[0].parse::<u8>()?;
        let denominator = parts[1].parse::<u8>()?;

        Ok(TimeSignature {
            numerator,
            denominator,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_signature() {
        let ts: TimeSignature = "4/4".parse().unwrap();
        assert_eq!(ts.numerator, 4);
        assert_eq!(ts.denominator, 4);
    }
}

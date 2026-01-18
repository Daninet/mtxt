use anyhow::{Result, anyhow};
use std::fmt;
use std::str::FromStr;

use crate::BeatTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeatFraction {
    numerator: u32,
    denominator: u32,
}

impl BeatFraction {
    pub fn new(numerator: u32, denominator: u32) -> Result<Self> {
        if denominator == 0 {
            return Err(anyhow!("Denominator cannot be zero"));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    pub fn as_beat_time(&self) -> BeatTime {
        let frac = self.numerator as f64 / self.denominator as f64;
        BeatTime::from_parts(frac.floor() as u32, frac.fract() as f32)
    }
}

impl FromStr for BeatFraction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid fraction format: {}", s));
        }

        let numerator: u32 = parts[0]
            .parse()
            .map_err(|_| anyhow!("Invalid numerator: {}", parts[0]))?;
        let denominator: u32 = parts[1]
            .parse()
            .map_err(|_| anyhow!("Invalid denominator: {}", parts[1]))?;

        Self::new(numerator, denominator)
    }
}

impl fmt::Display for BeatFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beat_fraction_parsing() {
        let f: BeatFraction = "1/4".parse().unwrap();
        assert_eq!(f.numerator, 1);
        assert_eq!(f.denominator, 4);
        assert_eq!(f.as_beat_time().to_string(), "0.25");

        let f: BeatFraction = "3/2".parse().unwrap();
        assert_eq!(f.as_beat_time().to_string(), "1.5");

        let f: BeatFraction = "10/1000".parse().unwrap();
        assert_eq!(f.as_beat_time().to_string(), "0.01");

        assert!("1".parse::<BeatFraction>().is_err());
        assert!("1.5".parse::<BeatFraction>().is_err());
        assert!("1.5/2".parse::<BeatFraction>().is_err());
        assert!("1/2.5".parse::<BeatFraction>().is_err());
        assert!("1/0".parse::<BeatFraction>().is_err());
        assert!("1//2".parse::<BeatFraction>().is_err());
        assert!("1 /2".parse::<BeatFraction>().is_err());
        assert!("1/ 2".parse::<BeatFraction>().is_err());
        assert!("-1/2".parse::<BeatFraction>().is_err());
        assert!("1/-2".parse::<BeatFraction>().is_err());
    }
}

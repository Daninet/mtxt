use anyhow::Result;
use anyhow::anyhow;
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

/// Beat-based time notation using fixed-point units
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct BeatTime {
    repr: u64,
}

impl BeatTime {
    /// Number of bits for the sub-beat units.
    const FRAC_BEAT_BITS: u32 = 32;
    /// The number of sub-units in a single beat (2^32).
    pub const FRAC_BEAT_COUNT: u64 = 1 << Self::FRAC_BEAT_BITS;
    /// Bitmask to extract the sub-unit part from the total units.
    const FRAC_BEAT_MASK: u64 = Self::FRAC_BEAT_COUNT - 1;

    pub const fn zero() -> Self {
        Self { repr: 0 }
    }

    const fn from_units(units: u64) -> Self {
        Self { repr: units }
    }

    pub fn from_parts(beat: u32, frac: f32) -> Self {
        assert!((0.0..=1.0).contains(&frac));
        if frac == 1.0 {
            return Self::from_parts(beat + 1, 0.0);
        }
        let frac_repr = (frac as f64 * Self::FRAC_BEAT_COUNT as f64) as u64;
        Self {
            repr: (beat as u64) << Self::FRAC_BEAT_BITS | frac_repr,
        }
    }

    pub fn as_f64(&self) -> f64 {
        self.repr_beat() as f64 + self.repr_frac_f32() as f64
    }

    pub fn as_micros(&self, bpm: f64) -> u64 {
        let micros_per_beat = 60_000_000.0 / bpm;
        (self.as_f64() * micros_per_beat).round() as u64
    }

    pub fn from_micros(micros: u64, bpm: f64) -> Self {
        let micros_per_beat = 60_000_000.0 / bpm;
        let beat = micros as f64 / micros_per_beat;
        let beat_int = beat.floor() as u32;
        let frac = beat.fract() as f32;
        Self::from_parts(beat_int, frac)
    }

    fn repr_beat(&self) -> u64 {
        self.repr >> Self::FRAC_BEAT_BITS
    }

    pub fn whole_beats(&self) -> u64 {
        self.repr_beat()
    }

    fn repr_frac(&self) -> u64 {
        self.repr & Self::FRAC_BEAT_MASK
    }

    fn repr_frac_f32(&self) -> f32 {
        (self.repr_frac() as f64 / Self::FRAC_BEAT_COUNT as f64) as f32
    }

    pub fn quantize(&self, grid: u32, swing: f32, humanize: f32) -> Self {
        if grid == 0 {
            return *self;
        }

        let grid_size = Self::FRAC_BEAT_COUNT as f64 / grid as f64;
        let total_sub_units = self.repr as f64;

        let mut quantized_units = if swing == 0.0 {
            (total_sub_units / grid_size).round() * grid_size
        } else {
            let grid_index = (total_sub_units / grid_size).round() as u32;
            let base_position = grid_index as f64 * grid_size;

            if grid_index.is_multiple_of(2) {
                // On-beat: snap to the main grid
                base_position
            } else {
                // Off-beat: apply swing
                // The "swing" factor moves the note from the straight 50% position
                // towards the classic triplet-feel 66.7% position.
                let swing_shift = (grid_size / 6.0) * swing as f64;
                base_position + swing_shift
            }
        };

        if humanize > 0.0 {
            // Humanize around the quantized position. The amount of randomization
            // is a quarter of the sub-grid size, scaled by the humanize factor.
            let sub_grid_size = grid_size / 2.0;
            let humanize_amount = sub_grid_size * 0.25 * humanize as f64;
            let humanize_offset = (rand::random::<f64>() - 0.5) * 2.0 * humanize_amount;
            quantized_units += humanize_offset;
        }

        Self::from_units(quantized_units.round() as u64)
    }
}

impl fmt::Display for BeatTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let beat = self.repr_beat();
        let frac_val = (self.repr_frac_f32() * 100_000.0).round() as u32;

        let mut frac = format!("{:05}", frac_val);
        while frac.ends_with('0') {
            frac.pop();
            if frac.is_empty() {
                frac.push('0');
                break;
            }
        }
        f.pad(&format!("{}.{}", beat, frac))
    }
}

impl fmt::Debug for BeatTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Time({})", self)
    }
}

impl Add for BeatTime {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::from_units(self.repr + other.repr)
    }
}

impl Sub for BeatTime {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::from_units(self.repr.saturating_sub(other.repr))
    }
}

impl FromStr for BeatTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();

        let mut parts = s.splitn(2, '.');
        let beat: u32 = parts
            .next()
            .ok_or_else(|| anyhow!("Invalid time: {}", s))?
            .parse()
            .map_err(|_e| anyhow!("Invalid time: {}", s))?;

        let frac_str = parts.next().unwrap_or("0");

        if !frac_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(anyhow!("Invalid time: {}", s));
        }

        let frac = format!("0.{}", frac_str)
            .parse()
            .map_err(|_e| anyhow!("Invalid time: {}", s))?;

        Ok(Self::from_parts(beat, frac))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let time: BeatTime = "4.123".parse().unwrap();
        assert_eq!(time.repr_beat(), 4);
        assert_eq!(time.repr_frac_f32(), 0.123);
        assert_eq!(time.to_string(), "4.123");

        assert_eq!("0".parse::<BeatTime>().unwrap().to_string(), "0.0");
        assert_eq!("0.".parse::<BeatTime>().unwrap().to_string(), "0.0");
        assert_eq!("0.0".parse::<BeatTime>().unwrap().to_string(), "0.0");
        assert_eq!("0.000".parse::<BeatTime>().unwrap().to_string(), "0.0");
        assert_eq!(" 7.25 ".parse::<BeatTime>().unwrap().to_string(), "7.25");
        assert_eq!(
            "0.99999".parse::<BeatTime>().unwrap().to_string(),
            "0.99999"
        );
        assert_eq!(
            "0.9999999999".parse::<BeatTime>().unwrap().to_string(),
            "1.0"
        );
        assert_eq!(
            "4294967295.99999".parse::<BeatTime>().unwrap().to_string(),
            "4294967295.99999"
        );

        // Test rounding
        assert_eq!(
            "0.123456".parse::<BeatTime>().unwrap().to_string(),
            "0.12346"
        );
        assert_eq!(
            "0.123454".parse::<BeatTime>().unwrap().to_string(),
            "0.12345"
        );
    }

    #[test]
    fn test_parse_error() {
        assert!("".parse::<BeatTime>().is_err());
        assert!("-0".parse::<BeatTime>().is_err());
        assert!("0x5".parse::<BeatTime>().is_err());
        assert!("-1.2".parse::<BeatTime>().is_err());
        assert!("2.3.4".parse::<BeatTime>().is_err());
        assert!("2.e5".parse::<BeatTime>().is_err());
        assert!("a".parse::<BeatTime>().is_err());
        assert!("4.9a".parse::<BeatTime>().is_err());
        assert!("1. 2".parse::<BeatTime>().is_err());
        assert!("1,2".parse::<BeatTime>().is_err());
        assert!("2.-3".parse::<BeatTime>().is_err());
    }

    #[test]
    fn test_op() {
        let time: BeatTime = "4.123".parse().unwrap();
        let other: BeatTime = "1.234".parse().unwrap();
        let sum = time + other;
        assert_eq!(sum.to_string(), "5.357");

        let diff = time - other;
        assert_eq!(diff.to_string(), "2.889");

        let overflow: BeatTime = "0.9".parse().unwrap();
        let sum = time + overflow;
        assert_eq!(sum.to_string(), "5.023");
    }

    #[test]
    fn test_quantize() {
        let time: BeatTime = "0.12".parse().unwrap();
        let quantized = time.quantize(4, 0.0, 0.0);
        assert_eq!(quantized.to_string(), "0.0"); // Quantized to the nearest 1/4 beat

        let time: BeatTime = "0.13".parse().unwrap();
        let quantized = time.quantize(4, 0.0, 0.0);
        assert_eq!(quantized.to_string(), "0.25"); // Quantized to the nearest 1/4 beat

        let time: BeatTime = "0.49".parse().unwrap();
        let quantized = time.quantize(4, 0.0, 0.0);
        assert_eq!(quantized.to_string(), "0.5");

        let time: BeatTime = "0.51".parse().unwrap();
        let quantized = time.quantize(4, 0.0, 0.0);
        assert_eq!(quantized.to_string(), "0.5");

        // Test with swing
        let time: BeatTime = "0.25".parse().unwrap(); // 0.25 is index 1 on grid=4 (0.25 spacing)
        let quantized = time.quantize(4, 1.0, 0.0);
        // 0.25 + (0.25/6) = 0.25 + 0.041666... = 0.29167
        assert_eq!(quantized.to_string(), "0.29167");

        // Test with humanize
        let time: BeatTime = "0.25".parse().unwrap();
        let quantized = time.quantize(4, 0.0, 0.5);
        assert!(quantized.to_string() != "0.25");
    }
}

use anyhow::{Result, bail};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PitchClass {
    Cb,
    C,
    CSharp,
    Db,
    D,
    DSharp,
    Eb,
    E,
    ESharp,
    Fb,
    F,
    FSharp,
    Gb,
    G,
    GSharp,
    Ab,
    A,
    ASharp,
    Bb,
    B,
    BSharp,
}

impl PitchClass {
    pub fn to_semitone(self) -> u8 {
        match self {
            PitchClass::BSharp | PitchClass::C => 0,
            PitchClass::CSharp | PitchClass::Db => 1,
            PitchClass::D => 2,
            PitchClass::DSharp | PitchClass::Eb => 3,
            PitchClass::Fb | PitchClass::E => 4,
            PitchClass::ESharp | PitchClass::F => 5,
            PitchClass::FSharp | PitchClass::Gb => 6,
            PitchClass::G => 7,
            PitchClass::GSharp | PitchClass::Ab => 8,
            PitchClass::A => 9,
            PitchClass::ASharp | PitchClass::Bb => 10,
            PitchClass::Cb | PitchClass::B => 11,
        }
    }

    pub fn to_canonical(self) -> Self {
        match self {
            PitchClass::Cb => PitchClass::B,
            PitchClass::Db => PitchClass::CSharp,
            PitchClass::DSharp => PitchClass::Eb,
            PitchClass::ESharp => PitchClass::F,
            PitchClass::Fb => PitchClass::E,
            PitchClass::Gb => PitchClass::FSharp,
            PitchClass::GSharp => PitchClass::Ab,
            PitchClass::ASharp => PitchClass::Bb,
            PitchClass::BSharp => PitchClass::C,
            _ => self,
        }
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PitchClass::Cb => "Cb",
            PitchClass::C => "C",
            PitchClass::CSharp => "C#",
            PitchClass::Db => "Db",
            PitchClass::D => "D",
            PitchClass::DSharp => "D#",
            PitchClass::Eb => "Eb",
            PitchClass::E => "E",
            PitchClass::ESharp => "E#",
            PitchClass::Fb => "Fb",
            PitchClass::F => "F",
            PitchClass::FSharp => "F#",
            PitchClass::Gb => "Gb",
            PitchClass::G => "G",
            PitchClass::GSharp => "G#",
            PitchClass::Ab => "Ab",
            PitchClass::A => "A",
            PitchClass::ASharp => "A#",
            PitchClass::Bb => "Bb",
            PitchClass::B => "B",
            PitchClass::BSharp => "B#",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for PitchClass {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_upper = s.to_uppercase();
        match s_upper.as_str() {
            "CB" => Ok(PitchClass::Cb),
            "C" => Ok(PitchClass::C),
            "C#" => Ok(PitchClass::CSharp),
            "DB" => Ok(PitchClass::Db),
            "D" => Ok(PitchClass::D),
            "D#" => Ok(PitchClass::DSharp),
            "EB" => Ok(PitchClass::Eb),
            "E" => Ok(PitchClass::E),
            "E#" => Ok(PitchClass::ESharp),
            "FB" => Ok(PitchClass::Fb),
            "F" => Ok(PitchClass::F),
            "F#" => Ok(PitchClass::FSharp),
            "GB" => Ok(PitchClass::Gb),
            "G" => Ok(PitchClass::G),
            "G#" => Ok(PitchClass::GSharp),
            "AB" => Ok(PitchClass::Ab),
            "A" => Ok(PitchClass::A),
            "A#" => Ok(PitchClass::ASharp),
            "BB" => Ok(PitchClass::Bb),
            "B" => Ok(PitchClass::B),
            "B#" => Ok(PitchClass::BSharp),
            _ => bail!("Invalid pitch class: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_class_parsing() {
        assert_eq!("C#".parse::<PitchClass>().unwrap(), PitchClass::CSharp);
        assert_eq!("Bb".parse::<PitchClass>().unwrap(), PitchClass::Bb);
        assert_eq!("Cb".parse::<PitchClass>().unwrap(), PitchClass::Cb);
    }

    #[test]
    fn test_case_insensitive_pitch_class_parsing() {
        assert_eq!("c".parse::<PitchClass>().unwrap(), PitchClass::C);
        assert_eq!("c#".parse::<PitchClass>().unwrap(), PitchClass::CSharp);
        assert_eq!("db".parse::<PitchClass>().unwrap(), PitchClass::Db);
        assert_eq!("bb".parse::<PitchClass>().unwrap(), PitchClass::Bb);

        assert_eq!("Bb".parse::<PitchClass>().unwrap(), PitchClass::Bb);
        assert_eq!("bB".parse::<PitchClass>().unwrap(), PitchClass::Bb);
        assert_eq!("F#".parse::<PitchClass>().unwrap(), PitchClass::FSharp);
        assert_eq!("f#".parse::<PitchClass>().unwrap(), PitchClass::FSharp);
    }
}

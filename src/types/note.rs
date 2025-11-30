use crate::PitchClass;
use crate::types::record::AliasDefinition;
use anyhow::{Result, bail};
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub pitch_class: PitchClass,
    pub octave: i8,
    pub cents: f32, // [-100.0 .. +100.0]
}

impl Note {
    pub fn new(pitch_class: PitchClass, octave: i8, cents: f32) -> Result<Self> {
        if !(-100.0..=100.0).contains(&cents) {
            bail!(
                "Cents deviation must be between -100.0 and +100.0, got {}",
                cents
            );
        }
        Ok(Self {
            pitch_class,
            octave,
            cents,
        })
    }

    /// Convert to MIDI note number (C4 = 60)
    pub fn to_midi_note(&self) -> u8 {
        let base = (self.octave + 1) * 12 + self.pitch_class.to_semitone() as i8;
        base.clamp(0, 127) as u8
    }

    pub fn transpose(&self, semitones: i32) -> Note {
        let current_semitone = self.pitch_class.to_semitone() as i32; // 0-11
        let current_abs = (self.octave as i32 + 1) * 12 + current_semitone;
        let new_abs = current_abs + semitones;

        let new_octave = new_abs.div_euclid(12) - 1;
        let new_semitone_idx = new_abs.rem_euclid(12);

        let new_pitch_class = match new_semitone_idx {
            0 => PitchClass::C,
            1 => PitchClass::CSharp,
            2 => PitchClass::D,
            3 => PitchClass::DSharp,
            4 => PitchClass::E,
            5 => PitchClass::F,
            6 => PitchClass::FSharp,
            7 => PitchClass::G,
            8 => PitchClass::GSharp,
            9 => PitchClass::A,
            10 => PitchClass::ASharp,
            11 => PitchClass::B,
            _ => unreachable!(),
        }
        .to_canonical();

        Note {
            pitch_class: new_pitch_class,
            octave: new_octave as i8,
            cents: self.cents,
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.pitch_class, self.octave)?;
        if self.cents != 0.0 {
            write!(f, "{:+}", self.cents)?;
        }
        Ok(())
    }
}

impl FromStr for Note {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Parse note like "C4", "D#5", "Bb3+50", "F4-25"
        let mut chars = s.chars().peekable();

        // Parse pitch class
        let pitch_char = chars
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty note string"))?;
        let mut pitch_str = pitch_char.to_string();

        // Check for accidental
        if let Some(&next_char) = chars.peek()
            && (next_char == '#' || next_char == 'b' || next_char == 'B') {
                pitch_str.push(chars.next().unwrap());
            }

        let pitch_class: PitchClass = pitch_str.parse()?;

        // Parse octave
        let remaining: String = chars.collect();

        // Find where octave ends and cents begin
        let mut octave_end = remaining.len();
        for (i, ch) in remaining.chars().enumerate() {
            if ch == '+' || ch == '-' {
                octave_end = i;
                break;
            }
        }

        let octave: i8 = remaining[..octave_end].parse()?;

        // Parse cents if present
        let cents = if octave_end < remaining.len() {
            remaining[octave_end..].parse()?
        } else {
            0.0f32
        };

        Self::new(pitch_class, octave, cents)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteTarget {
    Note(Note),
    AliasKey(String),
    Alias(Rc<AliasDefinition>),
}

impl fmt::Display for NoteTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteTarget::Note(note) => write!(f, "{}", note),
            NoteTarget::AliasKey(alias) => write!(f, "{}", alias),
            NoteTarget::Alias(alias) => write!(f, "{}", alias.name),
        }
    }
}

impl FromStr for NoteTarget {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // try to parse as note
        if let Ok(note) = s.parse::<Note>() {
            return Ok(NoteTarget::Note(note));
        }

        Ok(NoteTarget::AliasKey(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_note_parsing() {
        let note: Note = "C4".parse().unwrap();
        assert_eq!(note.pitch_class, PitchClass::C);
        assert_eq!(note.octave, 4);
        assert_eq!(note.cents, 0.0);
        assert_eq!(note.to_midi_note(), 60);
    }

    #[test]
    fn pitch() {
        assert_eq!("C0".parse::<Note>().unwrap().pitch_class, PitchClass::C);
        assert_eq!(
            "C#0".parse::<Note>().unwrap().pitch_class,
            PitchClass::CSharp
        );
        assert_eq!("Cb0".parse::<Note>().unwrap().pitch_class, PitchClass::Cb);
        assert_eq!("Db0".parse::<Note>().unwrap().pitch_class, PitchClass::Db);
    }

    #[test]
    fn cents() {
        assert_eq!("D4+50.5".parse::<Note>().unwrap().cents, 50.5);
        assert_eq!("D4-50".parse::<Note>().unwrap().cents, -50.0);
        assert_eq!("D4".parse::<Note>().unwrap().cents, 0.0);
        assert_eq!("D4-0".parse::<Note>().unwrap().cents, 0.0);
        assert_eq!("D4+0".parse::<Note>().unwrap().cents, 0.0);
    }
}

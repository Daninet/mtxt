use crate::types::note::Note;
use crate::types::pitch::PitchClass;
use anyhow::{Result, anyhow};

const MIDI_CC_MAPPINGS: &[(u8, &str)] = &[
    (1, "vibrato"),
    (2, "breath"),
    (4, "foot"),
    (5, "portamento"),
    (7, "volume"),
    (8, "balance"),
    (10, "pan"),
    (11, "expression"),
    (64, "sustain"),
    (65, "portamento_switch"),
    (66, "sostenuto"),
    (67, "soft"),
    (68, "legato"),
    (70, "sound_variation"),
    (71, "timbre"),
    (73, "attack"),
    (74, "cutoff"),
    (75, "decay"),
    (76, "vibrato_rate"),
    (77, "vibrato_depth"),
    (78, "vibrato_delay"),
    (91, "reverb"),
    (92, "tremolo"),
    (93, "chorus"),
    (94, "detune"),
    (95, "phaser"),
];

pub fn midi_cc_to_name(cc: u8) -> String {
    MIDI_CC_MAPPINGS
        .iter()
        .find(|(num, _)| *num == cc)
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| cc.to_string())
}

fn midi_cc_name_to_number(name: &str) -> Option<u8> {
    MIDI_CC_MAPPINGS
        .iter()
        .find(|(_, n)| *n == name)
        .map(|(num, _)| *num)
}

pub fn note_to_midi_number(note: &Note) -> Result<u8> {
    let pitch_offset = match note.pitch_class {
        PitchClass::C => 0,
        PitchClass::CSharp | PitchClass::Db => 1,
        PitchClass::D => 2,
        PitchClass::DSharp | PitchClass::Eb => 3,
        PitchClass::E => 4,
        PitchClass::F => 5,
        PitchClass::FSharp | PitchClass::Gb => 6,
        PitchClass::G => 7,
        PitchClass::GSharp | PitchClass::Ab => 8,
        PitchClass::A => 9,
        PitchClass::ASharp | PitchClass::Bb => 10,
        PitchClass::B => 11,
        _ => note.pitch_class.to_semitone(),
    };

    let midi_number = (note.octave as i32 + 1) * 12 + pitch_offset as i32;

    if !(0..=127).contains(&midi_number) {
        anyhow::bail!("Note {} is outside MIDI range", note);
    }

    Ok(midi_number as u8)
}

pub fn midi_key_to_note(key: u8) -> Result<Note> {
    let octave = (key / 12) as i8 - 1;
    let pitch_class = match key % 12 {
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
    };

    Note::new(pitch_class, octave, 0.0)
}

pub enum MidiControllerEvent {
    CC { number: u8, value: u8 },
    PitchBend { value: u16 },
    Aftertouch { value: u8 },
}

pub fn controller_name_to_midi(name: &str, value: f32) -> Result<MidiControllerEvent> {
    match name {
        "pitch" => {
            // Clamp to reasonable range
            let clamped = value.clamp(-12.0, 12.0);
            // Map from -12..12 to 0..16383
            // 0 (no bend) = 8192
            let bend_val = ((clamped + 12.0) / 24.0 * 16383.0) as u16;
            Ok(MidiControllerEvent::PitchBend { value: bend_val })
        }

        "aftertouch" => {
            let val = (value.clamp(0.0, 1.0) * 127.0) as u8;
            Ok(MidiControllerEvent::Aftertouch { value: val })
        }

        // Special aliases that need different handling
        "balance" => Ok(MidiControllerEvent::CC {
            number: midi_cc_name_to_number("balance").unwrap(),
            value: ((value.clamp(-1.0, 1.0) + 1.0) / 2.0 * 127.0) as u8,
        }),
        "pan" => Ok(MidiControllerEvent::CC {
            number: midi_cc_name_to_number("pan").unwrap(),
            value: ((value.clamp(-1.0, 1.0) + 1.0) / 2.0 * 127.0) as u8,
        }),

        // Aliases for standard CC names
        "resonance" => Ok(MidiControllerEvent::CC {
            number: midi_cc_name_to_number("timbre").unwrap(),
            value: (value.clamp(0.0, 1.0) * 127.0) as u8,
        }),
        "brightness" => Ok(MidiControllerEvent::CC {
            number: midi_cc_name_to_number("cutoff").unwrap(),
            value: (value.clamp(0.0, 1.0) * 127.0) as u8,
        }),

        // Try standard CC names from centralized mapping
        _ => {
            // Try to find a standard CC name
            if let Some(cc_num) = midi_cc_name_to_number(name) {
                return Ok(MidiControllerEvent::CC {
                    number: cc_num,
                    value: (value.clamp(0.0, 1.0) * 127.0) as u8,
                });
            }

            // Try parsing as a numeric CC number
            if let Ok(num) = name.parse::<u8>()
                && num <= 127 {
                    return Ok(MidiControllerEvent::CC {
                        number: num,
                        value: (value.clamp(0.0, 1.0) * 127.0) as u8,
                    });
                }

            Err(anyhow!("Unknown controller name: {}", name))
        }
    }
}

pub fn time_signature_to_midi(sig: &crate::types::time_signature::TimeSignature) -> (u8, u8) {
    // MIDI format: numerator, log2(denominator)
    // For example, 4/4 -> (4, 2) because 2^2 = 4
    let denom_log2 = (sig.denominator as f32).log2() as u8;
    (sig.numerator, denom_log2)
}

pub fn midi_key_signature_to_string(sharps_flats: i8, minor: bool) -> String {
    let key_name = match (sharps_flats, minor) {
        // Major keys (minor = false)
        (0, false) => "C",
        (1, false) => "G",
        (2, false) => "D",
        (3, false) => "A",
        (4, false) => "E",
        (5, false) => "B",
        (6, false) => "F#",
        (7, false) => "C#",
        (-1, false) => "F",
        (-2, false) => "Bb",
        (-3, false) => "Eb",
        (-4, false) => "Ab",
        (-5, false) => "Db",
        (-6, false) => "Gb",
        (-7, false) => "Cb",

        // Minor keys (minor = true)
        (0, true) => "A",
        (1, true) => "E",
        (2, true) => "B",
        (3, true) => "F#",
        (4, true) => "C#",
        (5, true) => "G#",
        (6, true) => "D#",
        (7, true) => "A#",
        (-1, true) => "D",
        (-2, true) => "G",
        (-3, true) => "C",
        (-4, true) => "F",
        (-5, true) => "Bb",
        (-6, true) => "Eb",
        (-7, true) => "Ab",

        // Fallback for out of range values
        _ => return format!("{} {}", sharps_flats, if minor { "minor" } else { "major" }),
    };

    format!("{} {}", key_name, if minor { "minor" } else { "major" })
}

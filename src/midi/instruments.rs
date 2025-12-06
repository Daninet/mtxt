#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instrument {
    pub gm_number: u8,
    pub gm_name: &'static str,
    pub mtxt_name: &'static str,
}

pub const INSTRUMENTS: [Instrument; 128] = [
    // Piano
    Instrument {
        gm_number: 0,
        gm_name: "Acoustic Grand Piano",
        mtxt_name: "piano_acoustic",
    },
    Instrument {
        gm_number: 1,
        gm_name: "Bright Acoustic Piano",
        mtxt_name: "piano_acoustic",
    },
    Instrument {
        gm_number: 2,
        gm_name: "Electric Grand Piano",
        mtxt_name: "piano_electric_grand",
    },
    Instrument {
        gm_number: 3,
        gm_name: "Honky-tonk Piano",
        mtxt_name: "piano_acoustic",
    },
    Instrument {
        gm_number: 4,
        gm_name: "Electric Piano 1",
        mtxt_name: "piano_electric",
    },
    Instrument {
        gm_number: 5,
        gm_name: "Electric Piano 2",
        mtxt_name: "piano_electric_tine",
    },
    Instrument {
        gm_number: 6,
        gm_name: "Harpsichord",
        mtxt_name: "harpsichord",
    },
    Instrument {
        gm_number: 7,
        gm_name: "Clavinet",
        mtxt_name: "clavinet",
    },
    // Chromatic Percussion
    Instrument {
        gm_number: 8,
        gm_name: "Celesta",
        mtxt_name: "glockenspiel",
    },
    Instrument {
        gm_number: 9,
        gm_name: "Glockenspiel",
        mtxt_name: "glockenspiel",
    },
    Instrument {
        gm_number: 10,
        gm_name: "Music Box",
        mtxt_name: "kalimba",
    },
    Instrument {
        gm_number: 11,
        gm_name: "Vibraphone",
        mtxt_name: "vibraphone",
    },
    Instrument {
        gm_number: 12,
        gm_name: "Marimba",
        mtxt_name: "marimba",
    },
    Instrument {
        gm_number: 13,
        gm_name: "Xylophone",
        mtxt_name: "xylophone",
    },
    Instrument {
        gm_number: 14,
        gm_name: "Tubular Bells",
        mtxt_name: "tubular_bells",
    },
    Instrument {
        gm_number: 15,
        gm_name: "Dulcimer",
        mtxt_name: "zither",
    },
    // Organ
    Instrument {
        gm_number: 16,
        gm_name: "Drawbar Organ",
        mtxt_name: "organ_tonewheel",
    },
    Instrument {
        gm_number: 17,
        gm_name: "Percussive Organ",
        mtxt_name: "organ_percussive",
    },
    Instrument {
        gm_number: 18,
        gm_name: "Rock Organ",
        mtxt_name: "organ_rock",
    },
    Instrument {
        gm_number: 19,
        gm_name: "Church Organ",
        mtxt_name: "organ_pipe",
    },
    Instrument {
        gm_number: 20,
        gm_name: "Reed Organ",
        mtxt_name: "organ_reed",
    },
    Instrument {
        gm_number: 21,
        gm_name: "Accordion",
        mtxt_name: "accordion",
    },
    Instrument {
        gm_number: 22,
        gm_name: "Harmonica",
        mtxt_name: "harmonica",
    },
    Instrument {
        gm_number: 23,
        gm_name: "Tango Accordion",
        mtxt_name: "accordion",
    },
    // Guitar
    Instrument {
        gm_number: 24,
        gm_name: "Acoustic Nylon Guitar",
        mtxt_name: "guitar_acoustic_nylon",
    },
    Instrument {
        gm_number: 25,
        gm_name: "Acoustic Steel Guitar",
        mtxt_name: "guitar_acoustic_steel",
    },
    Instrument {
        gm_number: 26,
        gm_name: "Electric Jazz Guitar",
        mtxt_name: "guitar_electric_jazz",
    },
    Instrument {
        gm_number: 27,
        gm_name: "Electric Clean Guitar",
        mtxt_name: "guitar_electric_clean",
    },
    Instrument {
        gm_number: 28,
        gm_name: "Electric Muted Guitar",
        mtxt_name: "guitar_electric_muted",
    },
    Instrument {
        gm_number: 29,
        gm_name: "Overdriven Guitar",
        mtxt_name: "guitar_electric_overdrive",
    },
    Instrument {
        gm_number: 30,
        gm_name: "Distortion Guitar",
        mtxt_name: "guitar_electric_overdrive",
    },
    Instrument {
        gm_number: 31,
        gm_name: "Guitar Harmonics",
        mtxt_name: "guitar",
    },
    // Bass
    Instrument {
        gm_number: 32,
        gm_name: "Acoustic Bass",
        mtxt_name: "bass_acoustic",
    },
    Instrument {
        gm_number: 33,
        gm_name: "Fingered Electric Bass",
        mtxt_name: "bass_electric",
    },
    Instrument {
        gm_number: 34,
        gm_name: "Plucked Electric Bass",
        mtxt_name: "bass_pick",
    },
    Instrument {
        gm_number: 35,
        gm_name: "Fretless Bass",
        mtxt_name: "bass_fretless",
    },
    Instrument {
        gm_number: 36,
        gm_name: "Slap Bass 1",
        mtxt_name: "bass_electric",
    },
    Instrument {
        gm_number: 37,
        gm_name: "Slap Bass 2",
        mtxt_name: "bass_electric",
    },
    Instrument {
        gm_number: 38,
        gm_name: "Synth Bass 1",
        mtxt_name: "bass_synth",
    },
    Instrument {
        gm_number: 39,
        gm_name: "Synth Bass 2",
        mtxt_name: "bass_synth",
    },
    // Strings
    Instrument {
        gm_number: 40,
        gm_name: "Violin",
        mtxt_name: "violin",
    },
    Instrument {
        gm_number: 41,
        gm_name: "Viola",
        mtxt_name: "viola",
    },
    Instrument {
        gm_number: 42,
        gm_name: "Cello",
        mtxt_name: "cello",
    },
    Instrument {
        gm_number: 43,
        gm_name: "Contrabass",
        mtxt_name: "contrabass",
    },
    Instrument {
        gm_number: 44,
        gm_name: "Tremolo Strings",
        mtxt_name: "strings",
    },
    Instrument {
        gm_number: 45,
        gm_name: "Pizzicato Strings",
        mtxt_name: "strings",
    },
    Instrument {
        gm_number: 46,
        gm_name: "Orchestral Harp",
        mtxt_name: "harp",
    },
    Instrument {
        gm_number: 47,
        gm_name: "Timpani",
        mtxt_name: "timpani",
    },
    // Ensemble
    Instrument {
        gm_number: 48,
        gm_name: "String Ensemble 1",
        mtxt_name: "strings",
    },
    Instrument {
        gm_number: 49,
        gm_name: "String Ensemble 2",
        mtxt_name: "strings",
    },
    Instrument {
        gm_number: 50,
        gm_name: "Synth Strings 1",
        mtxt_name: "strings_synth",
    },
    Instrument {
        gm_number: 51,
        gm_name: "Synth Strings 2",
        mtxt_name: "strings_synth",
    },
    Instrument {
        gm_number: 52,
        gm_name: "Choir Aahs",
        mtxt_name: "voice_solo",
    },
    Instrument {
        gm_number: 53,
        gm_name: "Voice Oohs",
        mtxt_name: "voice_solo",
    },
    Instrument {
        gm_number: 54,
        gm_name: "Synth Voice",
        mtxt_name: "voice",
    },
    Instrument {
        gm_number: 55,
        gm_name: "Orchestral Hit",
        mtxt_name: "drums_orchestral",
    },
    // Brass
    Instrument {
        gm_number: 56,
        gm_name: "Trumpet",
        mtxt_name: "trumpet",
    },
    Instrument {
        gm_number: 57,
        gm_name: "Trombone",
        mtxt_name: "trombone",
    },
    Instrument {
        gm_number: 58,
        gm_name: "Tuba",
        mtxt_name: "tuba",
    },
    Instrument {
        gm_number: 59,
        gm_name: "Muted Trumpet",
        mtxt_name: "trumpet",
    },
    Instrument {
        gm_number: 60,
        gm_name: "French Horn",
        mtxt_name: "french_horn",
    },
    Instrument {
        gm_number: 61,
        gm_name: "Brass Section",
        mtxt_name: "brass",
    },
    Instrument {
        gm_number: 62,
        gm_name: "Synth Brass 1",
        mtxt_name: "brass_synth",
    },
    Instrument {
        gm_number: 63,
        gm_name: "Synth Brass 2",
        mtxt_name: "brass_synth",
    },
    // Reed Timbres
    Instrument {
        gm_number: 64,
        gm_name: "Soprano Sax",
        mtxt_name: "sax_soprano",
    },
    Instrument {
        gm_number: 65,
        gm_name: "Alto Sax",
        mtxt_name: "sax_alto",
    },
    Instrument {
        gm_number: 66,
        gm_name: "Tenor Sax",
        mtxt_name: "sax_tenor",
    },
    Instrument {
        gm_number: 67,
        gm_name: "Baritone Sax",
        mtxt_name: "sax_baritone",
    },
    Instrument {
        gm_number: 68,
        gm_name: "Oboe",
        mtxt_name: "oboe",
    },
    Instrument {
        gm_number: 69,
        gm_name: "English Horn",
        mtxt_name: "english_horn",
    },
    Instrument {
        gm_number: 70,
        gm_name: "Bassoon",
        mtxt_name: "bassoon",
    },
    Instrument {
        gm_number: 71,
        gm_name: "Clarinet",
        mtxt_name: "clarinet",
    },
    // Pipe
    Instrument {
        gm_number: 72,
        gm_name: "Piccolo",
        mtxt_name: "piccolo",
    },
    Instrument {
        gm_number: 73,
        gm_name: "Flute",
        mtxt_name: "flute",
    },
    Instrument {
        gm_number: 74,
        gm_name: "Recorder",
        mtxt_name: "recorder",
    },
    Instrument {
        gm_number: 75,
        gm_name: "Pan Flute",
        mtxt_name: "pan_flute",
    },
    Instrument {
        gm_number: 76,
        gm_name: "Bottle Blow",
        mtxt_name: "flute",
    },
    Instrument {
        gm_number: 77,
        gm_name: "Shakuhachi",
        mtxt_name: "flute",
    },
    Instrument {
        gm_number: 78,
        gm_name: "Whistle",
        mtxt_name: "whistle",
    },
    Instrument {
        gm_number: 79,
        gm_name: "Ocarina",
        mtxt_name: "flute",
    },
    // Synth Lead
    Instrument {
        gm_number: 80,
        gm_name: "Square Wave Lead",
        mtxt_name: "synth_lead_square",
    },
    Instrument {
        gm_number: 81,
        gm_name: "Sawtooth Wave Lead",
        mtxt_name: "synth_lead_saw",
    },
    Instrument {
        gm_number: 82,
        gm_name: "Calliope Lead",
        mtxt_name: "synth_lead",
    },
    Instrument {
        gm_number: 83,
        gm_name: "Chiff Lead",
        mtxt_name: "synth_lead",
    },
    Instrument {
        gm_number: 84,
        gm_name: "Charang Lead",
        mtxt_name: "synth_lead",
    },
    Instrument {
        gm_number: 85,
        gm_name: "Voice Lead",
        mtxt_name: "voice_solo",
    },
    Instrument {
        gm_number: 86,
        gm_name: "Fifths Lead",
        mtxt_name: "synth_lead",
    },
    Instrument {
        gm_number: 87,
        gm_name: "Bass Lead",
        mtxt_name: "synth_lead",
    },
    // Synth Pad
    Instrument {
        gm_number: 88,
        gm_name: "New Age Pad",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 89,
        gm_name: "Warm Pad",
        mtxt_name: "synth_pad_warm",
    },
    Instrument {
        gm_number: 90,
        gm_name: "Polysynth Pad",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 91,
        gm_name: "Choir Pad",
        mtxt_name: "synth_pad_choir",
    },
    Instrument {
        gm_number: 92,
        gm_name: "Bowed Pad",
        mtxt_name: "synth_pad_glass",
    },
    Instrument {
        gm_number: 93,
        gm_name: "Metallic Pad",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 94,
        gm_name: "Halo Pad",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 95,
        gm_name: "Sweep Pad",
        mtxt_name: "synth_pad",
    },
    // Synth Effects
    Instrument {
        gm_number: 96,
        gm_name: "Rain Effect",
        mtxt_name: "shaker",
    },
    Instrument {
        gm_number: 97,
        gm_name: "Soundtrack Effect",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 98,
        gm_name: "Crystal Effect",
        mtxt_name: "glockenspiel",
    },
    Instrument {
        gm_number: 99,
        gm_name: "Atmosphere Effect",
        mtxt_name: "guitar",
    },
    Instrument {
        gm_number: 100,
        gm_name: "Brightness Effect",
        mtxt_name: "guitar",
    },
    Instrument {
        gm_number: 101,
        gm_name: "Goblins Effect",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 102,
        gm_name: "Echoes Effect",
        mtxt_name: "synth_pad",
    },
    Instrument {
        gm_number: 103,
        gm_name: "Sci-Fi Effect",
        mtxt_name: "synth_pad",
    },
    // Ethnic
    Instrument {
        gm_number: 104,
        gm_name: "Sitar",
        mtxt_name: "sitar",
    },
    Instrument {
        gm_number: 105,
        gm_name: "Banjo",
        mtxt_name: "banjo",
    },
    Instrument {
        gm_number: 106,
        gm_name: "Shamisen",
        mtxt_name: "sitar",
    },
    Instrument {
        gm_number: 107,
        gm_name: "Koto",
        mtxt_name: "zither",
    },
    Instrument {
        gm_number: 108,
        gm_name: "Kalimba",
        mtxt_name: "kalimba",
    },
    Instrument {
        gm_number: 109,
        gm_name: "Bagpipe",
        mtxt_name: "bagpipe",
    },
    Instrument {
        gm_number: 110,
        gm_name: "Fiddle",
        mtxt_name: "violin",
    },
    Instrument {
        gm_number: 111,
        gm_name: "Shanai",
        mtxt_name: "oboe",
    },
    // Percussive
    Instrument {
        gm_number: 112,
        gm_name: "Tinkle Bell",
        mtxt_name: "glockenspiel",
    },
    Instrument {
        gm_number: 113,
        gm_name: "Agogo",
        mtxt_name: "woodblock",
    },
    Instrument {
        gm_number: 114,
        gm_name: "Steel Drums",
        mtxt_name: "steelpan",
    },
    Instrument {
        gm_number: 115,
        gm_name: "Woodblock",
        mtxt_name: "woodblock",
    },
    Instrument {
        gm_number: 116,
        gm_name: "Taiko Drum",
        mtxt_name: "taiko",
    },
    Instrument {
        gm_number: 117,
        gm_name: "Melodic Tom",
        mtxt_name: "melodic_tom",
    },
    Instrument {
        gm_number: 118,
        gm_name: "Synth Drum",
        mtxt_name: "drums_electronic",
    },
    Instrument {
        gm_number: 119,
        gm_name: "Reverse Cymbal",
        mtxt_name: "cymbals",
    },
    // Sound Effects
    Instrument {
        gm_number: 120,
        gm_name: "Guitar Fret Noise",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 121,
        gm_name: "Breath Noise",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 122,
        gm_name: "Seashore",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 123,
        gm_name: "Bird Tweet",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 124,
        gm_name: "Telephone Ring",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 125,
        gm_name: "Helicopter",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 126,
        gm_name: "Applause",
        mtxt_name: "silence",
    },
    Instrument {
        gm_number: 127,
        gm_name: "Gunshot",
        mtxt_name: "silence",
    },
];

pub fn gm_to_mtxt(gm: u8) -> Option<&'static str> {
    INSTRUMENTS.get(gm as usize).map(|i| i.mtxt_name)
}

pub fn mtxt_to_gm(name: &str) -> Option<u8> {
    if let Some(instrument) = INSTRUMENTS.iter().find(|i| i.mtxt_name == name) {
        return Some(instrument.gm_number);
    }

    None
}

pub fn get_gm_name(gm: u8) -> Option<&'static str> {
    INSTRUMENTS.get(gm as usize).map(|i| i.gm_name)
}

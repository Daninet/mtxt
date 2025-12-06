#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drum {
    pub number: u8,
    pub name: &'static str,
    pub slug: &'static str,
}

pub const DRUMS: [Drum; 47] = [
    Drum {
        number: 35,
        name: "Acoustic Bass Drum",
        slug: "acoustic_bass_drum",
    },
    Drum {
        number: 36,
        name: "Bass Drum 1",
        slug: "bass_drum_1",
    },
    Drum {
        number: 37,
        name: "Side Stick",
        slug: "side_stick",
    },
    Drum {
        number: 38,
        name: "Acoustic Snare",
        slug: "acoustic_snare",
    },
    Drum {
        number: 39,
        name: "Hand Clap",
        slug: "hand_clap",
    },
    Drum {
        number: 40,
        name: "Electric Snare",
        slug: "electric_snare",
    },
    Drum {
        number: 41,
        name: "Low Floor Tom",
        slug: "low_floor_tom",
    },
    Drum {
        number: 42,
        name: "Closed High Hat",
        slug: "closed_high_hat",
    },
    Drum {
        number: 43,
        name: "High Floor Tom",
        slug: "high_floor_tom",
    },
    Drum {
        number: 44,
        name: "Pedal High Hat",
        slug: "pedal_high_hat",
    },
    Drum {
        number: 45,
        name: "Low Tom",
        slug: "low_tom",
    },
    Drum {
        number: 46,
        name: "Open High Hat",
        slug: "open_high_hat",
    },
    Drum {
        number: 47,
        name: "Low Mid Tom",
        slug: "low_mid_tom",
    },
    Drum {
        number: 48,
        name: "High Mid Tom",
        slug: "high_mid_tom",
    },
    Drum {
        number: 49,
        name: "Crash Cymbal 1",
        slug: "crash_cymbal_1",
    },
    Drum {
        number: 50,
        name: "High Tom",
        slug: "high_tom",
    },
    Drum {
        number: 51,
        name: "Ride Cymbal 1",
        slug: "ride_cymbal_1",
    },
    Drum {
        number: 52,
        name: "Chinese Cymbal",
        slug: "chinese_cymbal",
    },
    Drum {
        number: 53,
        name: "Ride Bell",
        slug: "ride_bell",
    },
    Drum {
        number: 54,
        name: "Tambourine",
        slug: "tambourine",
    },
    Drum {
        number: 55,
        name: "Splash Cymbal",
        slug: "splash_cymbal",
    },
    Drum {
        number: 56,
        name: "Cowbell",
        slug: "cowbell",
    },
    Drum {
        number: 57,
        name: "Crash Cymbal 2",
        slug: "crash_cymbal_2",
    },
    Drum {
        number: 58,
        name: "Vibraslap",
        slug: "vibraslap",
    },
    Drum {
        number: 59,
        name: "Ride Cymbal 2",
        slug: "ride_cymbal_2",
    },
    Drum {
        number: 60,
        name: "High Bongo",
        slug: "high_bongo",
    },
    Drum {
        number: 61,
        name: "Low Bongo",
        slug: "low_bongo",
    },
    Drum {
        number: 62,
        name: "Mute High Conga",
        slug: "mute_high_conga",
    },
    Drum {
        number: 63,
        name: "Open High Conga",
        slug: "open_high_conga",
    },
    Drum {
        number: 64,
        name: "Low Conga",
        slug: "low_conga",
    },
    Drum {
        number: 65,
        name: "High Timbale",
        slug: "high_timbale",
    },
    Drum {
        number: 66,
        name: "Low Timbale",
        slug: "low_timbale",
    },
    Drum {
        number: 67,
        name: "High Agogo",
        slug: "high_agogo",
    },
    Drum {
        number: 68,
        name: "Low Agogo",
        slug: "low_agogo",
    },
    Drum {
        number: 69,
        name: "Cabasa",
        slug: "cabasa",
    },
    Drum {
        number: 70,
        name: "Maracas",
        slug: "maracas",
    },
    Drum {
        number: 71,
        name: "Short Whistle",
        slug: "short_whistle",
    },
    Drum {
        number: 72,
        name: "Long Whistle",
        slug: "long_whistle",
    },
    Drum {
        number: 73,
        name: "Short Guiro",
        slug: "short_guiro",
    },
    Drum {
        number: 74,
        name: "Long Guiro",
        slug: "long_guiro",
    },
    Drum {
        number: 75,
        name: "Claves",
        slug: "claves",
    },
    Drum {
        number: 76,
        name: "High Wood Block",
        slug: "high_wood_block",
    },
    Drum {
        number: 77,
        name: "Low Wood Block",
        slug: "low_wood_block",
    },
    Drum {
        number: 78,
        name: "Mute Cuica",
        slug: "mute_cuica",
    },
    Drum {
        number: 79,
        name: "Open Cuica",
        slug: "open_cuica",
    },
    Drum {
        number: 80,
        name: "Mute Triangle",
        slug: "mute_triangle",
    },
    Drum {
        number: 81,
        name: "Open Triangle",
        slug: "open_triangle",
    },
];

pub fn get_drum_by_number(number: u8) -> Option<&'static Drum> {
    DRUMS.iter().find(|d| d.number == number)
}

pub fn get_drum_by_slug(slug: &str) -> Option<&'static Drum> {
    DRUMS.iter().find(|d| d.slug == slug)
}

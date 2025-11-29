use crate::Note;
use crate::TimeSignature;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MtxtOutputRecord {
    NoteOn {
        time: u64,
        note: Note,
        velocity: f32,
        channel: u16,
    },
    NoteOff {
        time: u64,
        note: Note,
        off_velocity: f32,
        channel: u16,
    },
    ControlChange {
        time: u64,
        note: Option<Note>,
        controller: String,
        value: f32,
        channel: u16,
    },
    Voice {
        time: u64,
        voices: Vec<String>,
        channel: u16,
    },
    Tempo {
        time: u64,
        bpm: f32,
    },
    TimeSignature {
        time: u64,
        signature: TimeSignature,
    },
    Reset {
        time: u64,
        target: String,
    },
    GlobalMeta {
        time: u64,
        meta_type: String,
        value: String,
    },
    ChannelMeta {
        time: u64,
        channel: u16,
        meta_type: String,
        value: String,
    },
    Beat {
        time: u64,
        beat: u64,
    },
    SysEx {
        time: u64,
        data: Vec<u8>,
    },
}

impl MtxtOutputRecord {
    pub fn time(&self) -> u64 {
        match self {
            MtxtOutputRecord::NoteOn { time, .. }
            | MtxtOutputRecord::NoteOff { time, .. }
            | MtxtOutputRecord::ControlChange { time, .. }
            | MtxtOutputRecord::Voice { time, .. }
            | MtxtOutputRecord::Tempo { time, .. }
            | MtxtOutputRecord::TimeSignature { time, .. }
            | MtxtOutputRecord::Reset { time, .. }
            | MtxtOutputRecord::GlobalMeta { time, .. }
            | MtxtOutputRecord::ChannelMeta { time, .. }
            | MtxtOutputRecord::SysEx { time, .. }
            | MtxtOutputRecord::Beat { time, .. } => *time,
        }
    }

    pub fn set_time(&mut self, micros: u64) {
        match self {
            MtxtOutputRecord::NoteOn { time, .. }
            | MtxtOutputRecord::NoteOff { time, .. }
            | MtxtOutputRecord::ControlChange { time, .. }
            | MtxtOutputRecord::Voice { time, .. }
            | MtxtOutputRecord::Tempo { time, .. }
            | MtxtOutputRecord::TimeSignature { time, .. }
            | MtxtOutputRecord::Reset { time, .. }
            | MtxtOutputRecord::GlobalMeta { time, .. }
            | MtxtOutputRecord::ChannelMeta { time, .. }
            | MtxtOutputRecord::SysEx { time, .. }
            | MtxtOutputRecord::Beat { time, .. } => *time = micros,
        };
    }

    // used for transitions
    pub fn get_parameter_value(&self) -> Option<f32> {
        match self {
            MtxtOutputRecord::ControlChange { value, .. } => Some(*value),
            MtxtOutputRecord::Tempo { bpm, .. } => Some(*bpm),
            _ => None,
        }
    }

    // used for transitions
    pub fn set_parameter_value(&mut self, val: f32) {
        match self {
            MtxtOutputRecord::ControlChange { value, .. } => *value = val,
            MtxtOutputRecord::Tempo { bpm, .. } => *bpm = val,
            _ => (),
        }
    }

    pub fn get_param_key(&self) -> Option<String> {
        match self {
            MtxtOutputRecord::ControlChange {
                channel,
                controller,
                ..
            } => Some(format!("cc:{}:{}", channel, controller)),
            MtxtOutputRecord::Tempo { .. } => Some("tempo".to_string()),
            _ => None,
        }
    }

    // used for aborting transitions
    pub fn is_same_parameter(&self, other: &MtxtOutputRecord) -> bool {
        match (self, other) {
            (
                MtxtOutputRecord::ControlChange {
                    channel: c1,
                    controller: cc1,
                    note: n1,
                    ..
                },
                MtxtOutputRecord::ControlChange {
                    channel: c2,
                    controller: cc2,
                    note: n2,
                    ..
                },
            ) => c1 == c2 && cc1 == cc2 && n1 == n2,
            (MtxtOutputRecord::Tempo { .. }, MtxtOutputRecord::Tempo { .. }) => true,
            _ => false,
        }
    }
}

impl fmt::Display for MtxtOutputRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format_time = |time: u64| format!("[{:>8}]", (time as f32 / 1000.0).round());

        let format_float = |val: f32| -> String {
            let rounded = (val * 1_000_000.0).round() / 1_000_000.0;
            let formatted = format!("{:.6}", rounded);
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        };

        match self {
            MtxtOutputRecord::NoteOn {
                time,
                note,
                velocity,
                channel,
            } => write!(
                f,
                "{} NoteOn {} vel={} ch={}",
                format_time(*time),
                note,
                format_float(*velocity),
                channel
            ),
            MtxtOutputRecord::NoteOff {
                time,
                note,
                off_velocity,
                channel,
            } => write!(
                f,
                "{} NoteOff {} off_vel={} ch={}",
                format_time(*time),
                note,
                format_float(*off_velocity),
                channel
            ),
            MtxtOutputRecord::ControlChange {
                time,
                note,
                controller,
                value,
                channel,
            } => {
                if let Some(n) = note {
                    write!(
                        f,
                        "{} CC {} {} val={} ch={}",
                        format_time(*time),
                        n,
                        controller,
                        format_float(*value),
                        channel
                    )
                } else {
                    write!(
                        f,
                        "{} CC {} val={} ch={}",
                        format_time(*time),
                        controller,
                        format_float(*value),
                        channel
                    )
                }
            }
            MtxtOutputRecord::Voice {
                time,
                voices,
                channel,
            } => write!(
                f,
                "{} Voice {:?} ch={}",
                format_time(*time),
                voices,
                channel
            ),
            MtxtOutputRecord::Tempo { time, bpm } => {
                write!(f, "{} Tempo bpm={}", format_time(*time), format_float(*bpm))
            }
            MtxtOutputRecord::TimeSignature { time, signature } => {
                write!(f, "{} TimeSignature {}", format_time(*time), signature)
            }

            MtxtOutputRecord::Reset { time, target } => {
                write!(f, "{} Reset {}", format_time(*time), target)
            }
            MtxtOutputRecord::GlobalMeta {
                time,
                meta_type,
                value,
            } => write!(
                f,
                "{} Meta global {} {}",
                format_time(*time),
                meta_type,
                value
            ),
            MtxtOutputRecord::ChannelMeta {
                time,
                channel,
                meta_type,
                value,
            } => write!(
                f,
                "{} Meta ch={} {} {}",
                format_time(*time),
                channel,
                meta_type,
                value
            ),
            MtxtOutputRecord::Beat { time, beat } => {
                write!(f, "{} Beat {}", format_time(*time), beat)
            }
            MtxtOutputRecord::SysEx { time, data } => {
                write!(f, "{} SysEx {:02X?}", format_time(*time), data)
            }
        }
    }
}

use crate::BeatTime;
use crate::Note;
use crate::TimeSignature;
use crate::Version;
use crate::types::note::NoteTarget;
use crate::util::format_float32;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct AliasDefinition {
    pub name: String,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MtxtRecord {
    Header {
        version: Version,
    },

    GlobalMeta {
        meta_type: String,
        value: String,
    },

    Meta {
        time: Option<BeatTime>,
        channel: Option<u16>, // channel might be defined by ChannelDirective
        meta_type: String,
        value: String,
    },

    DurationDirective {
        duration: BeatTime,
    },
    ChannelDirective {
        channel: u16,
    },
    VelocityDirective {
        velocity: f32,
    },
    OffVelocityDirective {
        off_velocity: f32,
    },
    TransitionCurveDirective {
        curve: f32,
    },
    TransitionIntervalDirective {
        interval: f32,
    },

    AliasDef {
        value: Rc<AliasDefinition>,
    },

    Note {
        time: BeatTime,
        note: NoteTarget,
        duration: Option<BeatTime>,
        velocity: Option<f32>,
        off_velocity: Option<f32>,
        channel: Option<u16>, // channel might be defined by ChannelDirective
    },
    NoteOn {
        time: BeatTime,
        note: NoteTarget,
        velocity: Option<f32>,
        channel: Option<u16>, // channel might be defined by ChannelDirective
    },
    NoteOff {
        time: BeatTime,
        note: NoteTarget,
        off_velocity: Option<f32>,
        channel: Option<u16>, // channel might be defined by ChannelDirective
    },

    ControlChange {
        time: BeatTime,
        note: Option<NoteTarget>,
        controller: String,
        value: f32,
        channel: Option<u16>, // if None, affect all channels
        transition_curve: Option<f32>,
        transition_time: Option<BeatTime>,
        transition_interval: Option<f32>,
    },
    Voice {
        time: BeatTime,
        voices: Vec<String>,
        channel: Option<u16>, // channel might be defined by ChannelDirective
    },

    Tempo {
        time: BeatTime,
        bpm: f32,
        transition_curve: Option<f32>,
        transition_time: Option<BeatTime>,
        transition_interval: Option<f32>,
    },
    TimeSignature {
        time: BeatTime,
        signature: TimeSignature,
    },

    Tuning {
        time: BeatTime,
        target: String,
        cents: f32,
    },
    Reset {
        time: BeatTime,
        target: String,
    },

    SysEx {
        time: BeatTime,
        data: Vec<u8>,
    },

    // Formatting events for passthrough conversion
    EmptyLine,
    Comment {
        text: String,
    },
}

impl fmt::Display for MtxtRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MtxtRecord::Header { version } => {
                write!(f, "mtxt {}", version)
            }
            MtxtRecord::GlobalMeta { meta_type, value } => {
                write!(f, "meta global {} {}", meta_type, value)
            }
            MtxtRecord::ChannelDirective { channel } => {
                write!(f, "ch={}", channel)
            }
            MtxtRecord::VelocityDirective { velocity } => {
                write!(f, "vel={}", format_float32(*velocity))
            }
            MtxtRecord::OffVelocityDirective { off_velocity } => {
                write!(f, "offvel={}", format_float32(*off_velocity))
            }
            MtxtRecord::DurationDirective { duration } => {
                write!(f, "dur={}", *duration)
            }
            MtxtRecord::TransitionCurveDirective { curve } => {
                write!(f, "transition_curve={}", format_float32(*curve))
            }
            MtxtRecord::TransitionIntervalDirective { interval } => {
                write!(f, "transition_interval={}", format_float32(*interval))
            }
            MtxtRecord::AliasDef { value } => {
                write!(f, "alias {}", value.name)?;
                for note in &value.notes {
                    write!(f, " {}", note)?;
                }
                Ok(())
            }
            MtxtRecord::Note {
                time: _,
                note,
                duration,
                velocity,
                off_velocity,
                channel,
            } => {
                write!(f, "note {}", note)?;
                if let Some(duration) = duration {
                    write!(f, " dur={}", *duration)?;
                }
                if let Some(vel) = velocity {
                    write!(f, " vel={}", format_float32(*vel))?;
                }
                if let Some(off_vel) = off_velocity {
                    write!(f, " offvel={}", format_float32(*off_vel))?;
                }
                if let Some(ch) = channel {
                    write!(f, " ch={}", ch)?;
                }
                Ok(())
            }
            MtxtRecord::NoteOn {
                time: _,
                note,
                velocity,
                channel,
            } => {
                write!(f, "on {}", note)?;
                if let Some(vel) = velocity {
                    write!(f, " vel={}", format_float32(*vel))?;
                }
                if let Some(ch) = channel {
                    write!(f, " ch={}", ch)?;
                }
                Ok(())
            }
            MtxtRecord::NoteOff {
                time: _,
                note,
                off_velocity,
                channel,
            } => {
                write!(f, "off {}", note)?;
                if let Some(off_vel) = off_velocity {
                    write!(f, " offvel={}", format_float32(*off_vel))?;
                }
                if let Some(ch) = channel {
                    write!(f, " ch={}", ch)?;
                }
                Ok(())
            }
            MtxtRecord::ControlChange {
                time: _,
                note,
                controller,
                value,
                channel,
                transition_curve,
                transition_time,
                transition_interval,
            } => {
                write!(f, "cc")?;
                if let Some(n) = note {
                    write!(f, " {}", n)?;
                }

                write!(f, " {} {}", controller, format_float32(*value))?;

                if let Some(ch) = channel {
                    write!(f, " ch={}", ch)?;
                }
                if let Some(curve) = transition_curve {
                    write!(f, " transition_curve={}", format_float32(*curve))?;
                }
                if let Some(time) = transition_time {
                    write!(f, " transition_time={}", *time)?;
                }
                if let Some(interval) = transition_interval {
                    write!(f, " transition_interval={}", format_float32(*interval))?;
                }
                Ok(())
            }
            MtxtRecord::Voice {
                time: _,
                voices,
                channel,
            } => {
                write!(f, "voice")?;
                if let Some(ch) = channel {
                    write!(f, " ch={}", ch)?;
                }
                if !voices.is_empty() {
                    write!(f, " {}", voices.join(", "))?;
                }
                Ok(())
            }
            MtxtRecord::Tempo {
                time: _,
                bpm,
                transition_curve,
                transition_time,
                transition_interval,
            } => {
                write!(f, "tempo {}", format_float32(*bpm))?;
                if let Some(curve) = transition_curve {
                    write!(f, " transition_curve={}", format_float32(*curve))?;
                }
                if let Some(time) = transition_time {
                    write!(f, " transition_time={}", *time)?;
                }
                if let Some(interval) = transition_interval {
                    write!(f, " transition_interval={}", format_float32(*interval))?;
                }
                Ok(())
            }
            MtxtRecord::TimeSignature { time: _, signature } => {
                write!(f, "timesig {}", signature)
            }
            MtxtRecord::Tuning {
                time: _,
                target,
                cents,
            } => {
                let s = format_float32(*cents);
                if *cents >= 0.0 && !s.starts_with('+') {
                    write!(f, "tuning {} +{}", target, s)
                } else {
                    write!(f, "tuning {} {}", target, s)
                }
            }
            MtxtRecord::Reset { time: _, target } => {
                write!(f, "reset {}", target)
            }
            MtxtRecord::Meta {
                time: _,
                channel,
                meta_type,
                value,
            } => {
                write!(f, "meta")?;
                if let Some(ch) = channel {
                    write!(f, " ch={}", ch)?;
                }
                write!(f, " {} {}", meta_type, value)
            }
            MtxtRecord::SysEx { time: _, data } => {
                write!(f, "sysex")?;
                for byte in data {
                    write!(f, " {:02x}", byte)?;
                }
                Ok(())
            }
            MtxtRecord::EmptyLine => {
                write!(f, "")
            }
            MtxtRecord::Comment { text, .. } => {
                write!(f, "// {}", text)
            }
        }
    }
}

impl MtxtRecord {
    pub fn time(&self) -> Option<BeatTime> {
        match self {
            MtxtRecord::Note { time, .. }
            | MtxtRecord::NoteOn { time, .. }
            | MtxtRecord::NoteOff { time, .. }
            | MtxtRecord::ControlChange { time, .. }
            | MtxtRecord::Tempo { time, .. }
            | MtxtRecord::TimeSignature { time, .. }
            | MtxtRecord::Voice { time, .. }
            | MtxtRecord::Tuning { time, .. }
            | MtxtRecord::Reset { time, .. }
            | MtxtRecord::SysEx { time, .. } => Some(*time),
            MtxtRecord::Meta { time, .. } => *time,
            _ => None,
        }
    }

    pub fn set_time(&mut self, t: BeatTime) {
        match self {
            MtxtRecord::Note { time, .. }
            | MtxtRecord::NoteOn { time, .. }
            | MtxtRecord::NoteOff { time, .. }
            | MtxtRecord::ControlChange { time, .. }
            | MtxtRecord::Tempo { time, .. }
            | MtxtRecord::TimeSignature { time, .. }
            | MtxtRecord::Voice { time, .. }
            | MtxtRecord::Tuning { time, .. }
            | MtxtRecord::Reset { time, .. }
            | MtxtRecord::SysEx { time, .. } => *time = t,
            MtxtRecord::Meta { time, .. } => *time = Some(t),
            _ => {}
        }
    }
}

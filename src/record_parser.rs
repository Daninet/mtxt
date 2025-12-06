use crate::types::record::VoiceList;
use crate::{
    BeatTime, MtxtRecord, Note, NoteTarget, TimeSignature, Version, types::record::AliasDefinition,
};
use anyhow::{Result, bail};
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
enum ParsedDirective {
    Channel { channel: u16 },
    Velocity { velocity: f32 },
    OffVelocity { off_velocity: f32 },
    Duration { duration: BeatTime },
    TransitionCurve { curve: f32 },
    TransitionTime { duration: BeatTime },
    TransitionInterval { interval: f32 },
}

impl fmt::Display for ParsedDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedDirective::Channel { channel } => write!(f, "ch={}", channel),
            ParsedDirective::Velocity { velocity } => write!(f, "vel={}", velocity),
            ParsedDirective::OffVelocity { off_velocity } => write!(f, "offvel={}", off_velocity),
            ParsedDirective::Duration { duration } => write!(f, "dur={}", duration),
            ParsedDirective::TransitionCurve { curve } => write!(f, "transition_curve={}", curve),
            ParsedDirective::TransitionTime { duration } => {
                write!(f, "transition_time={}", duration)
            }
            ParsedDirective::TransitionInterval { interval } => {
                write!(f, "transition_interval={}", interval)
            }
        }
    }
}

fn try_parse_directive(part: &str) -> Result<Option<ParsedDirective>> {
    let splitted = part.split_once("=");
    if let Some((key, value)) = splitted {
        match key {
            "ch" => {
                let channel: u16 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid channel number"))?;
                Ok(Some(ParsedDirective::Channel { channel }))
            }
            "vel" => {
                let velocity: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid velocity value"))?;
                if !(0.0..=1.0).contains(&velocity) {
                    bail!("Velocity must be 0.0-1.0");
                }
                Ok(Some(ParsedDirective::Velocity { velocity }))
            }
            "offvel" => {
                let off_velocity: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid off velocity value"))?;
                if !(0.0..=1.0).contains(&off_velocity) {
                    bail!("Off velocity must be 0.0-1.0");
                }
                Ok(Some(ParsedDirective::OffVelocity { off_velocity }))
            }
            "dur" => {
                let duration: BeatTime = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid duration value"))?;
                Ok(Some(ParsedDirective::Duration { duration }))
            }
            "transition_curve" => {
                let curve: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid transition_curve value"))?;
                Ok(Some(ParsedDirective::TransitionCurve { curve }))
            }
            "transition_time" => {
                let time: BeatTime = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid transition_time value"))?;
                Ok(Some(ParsedDirective::TransitionTime { duration: time }))
            }
            "transition_interval" => {
                let interval: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid transition_interval value"))?;
                if interval < 0.0 {
                    bail!("Transition interval must be >= 0.0");
                }
                Ok(Some(ParsedDirective::TransitionInterval { interval }))
            }
            _ => bail!("Invalid directive"),
        }
    } else {
        Ok(None)
    }
}

fn try_parse_global_directive(part: &str) -> Result<Option<MtxtRecord>> {
    let parsed = try_parse_directive(part)?;
    if let Some(parsed) = parsed {
        match parsed {
            ParsedDirective::Channel { channel } => {
                Ok(Some(MtxtRecord::ChannelDirective { channel }))
            }
            ParsedDirective::Velocity { velocity } => {
                Ok(Some(MtxtRecord::VelocityDirective { velocity }))
            }
            ParsedDirective::OffVelocity { off_velocity } => {
                Ok(Some(MtxtRecord::OffVelocityDirective { off_velocity }))
            }
            ParsedDirective::Duration { duration } => {
                Ok(Some(MtxtRecord::DurationDirective { duration }))
            }
            ParsedDirective::TransitionCurve { curve } => {
                Ok(Some(MtxtRecord::TransitionCurveDirective { curve }))
            }
            ParsedDirective::TransitionInterval { interval } => {
                Ok(Some(MtxtRecord::TransitionIntervalDirective { interval }))
            }
            ParsedDirective::TransitionTime {
                duration: _duration,
            } => {
                bail!("transition_time= is not supported here");
            }
        }
    } else {
        Ok(None)
    }
}

fn parse_note_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.is_empty() {
        bail!("Note event requires note name");
    }

    let note: NoteTarget = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid note"))?;

    let mut duration = None;
    let mut velocity = None;
    let mut off_velocity = None;
    let mut channel = None;

    for part in &parts[1..] {
        let directive = try_parse_directive(part);
        match directive {
            Ok(d) => match d {
                Some(ParsedDirective::Duration { duration: d }) => {
                    duration = Some(d);
                }
                Some(ParsedDirective::Velocity { velocity: v }) => {
                    velocity = Some(v);
                }
                Some(ParsedDirective::OffVelocity { off_velocity: v }) => {
                    off_velocity = Some(v);
                }
                Some(ParsedDirective::Channel { channel: c }) => {
                    channel = Some(c);
                }
                _ => bail!("Unsupported directive \"{}\"", part),
            },
            Err(e) => bail!("{}", e),
        }
    }

    Ok(MtxtRecord::Note {
        time,
        note,
        duration,
        velocity,
        off_velocity,
        channel,
    })
}

fn parse_note_on_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.is_empty() {
        bail!("Note on event requires note name");
    }

    let note: NoteTarget = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid note"))?;

    let mut velocity = None;
    let mut channel = None;

    for part in &parts[1..] {
        let directive = try_parse_directive(part);
        match directive {
            Ok(d) => match d {
                Some(ParsedDirective::Velocity { velocity: v }) => {
                    velocity = Some(v);
                }
                Some(ParsedDirective::Channel { channel: c }) => {
                    channel = Some(c);
                }
                _ => bail!("Unsupported directive \"{}\"", part),
            },
            Err(e) => bail!("{}", e),
        }
    }

    Ok(MtxtRecord::NoteOn {
        time,
        note,
        velocity,
        channel,
    })
}

fn parse_note_off_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.is_empty() {
        bail!("Note off event requires note name");
    }

    let note: NoteTarget = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid note"))?;

    let mut velocity = None;
    let mut channel = None;

    for part in &parts[1..] {
        let directive = try_parse_directive(part);
        match directive {
            Ok(d) => match d {
                Some(ParsedDirective::OffVelocity { off_velocity: v }) => {
                    velocity = Some(v);
                }
                Some(ParsedDirective::Channel { channel: c }) => {
                    channel = Some(c);
                }
                _ => bail!("Unsupported directive \"{}\"", part),
            },
            Err(e) => bail!("{}", e),
        }
    }

    Ok(MtxtRecord::NoteOff {
        time,
        note,
        off_velocity: velocity,
        channel,
    })
}

fn parse_control_change_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    let (note, controller, value, idx) = if parts.len() >= 3 && parts[2].parse::<f32>().is_ok() {
        // Case: cc <note> <controller> <value>
        let note: NoteTarget = parts[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid note"))?;
        let controller = parts[1].to_string();
        let value: f32 = parts[2].parse().unwrap();
        (Some(note), controller, value, 3)
    } else if parts.len() >= 2 && parts[1].parse::<f32>().is_ok() {
        // Case: cc <controller> <value>
        let controller = parts[0].to_string();
        let value: f32 = parts[1].parse().unwrap();
        (None, controller, value, 2)
    } else {
        bail!("CC event requires controller and value (float)");
    };

    let mut channel = None;
    let mut transition_curve = None;
    let mut transition_time = None;
    let mut transition_interval = None;

    for part in &parts[idx..] {
        let directive = try_parse_directive(part);
        match directive {
            Ok(d) => match d {
                Some(ParsedDirective::Channel { channel: c }) => {
                    channel = Some(c);
                }
                Some(ParsedDirective::TransitionCurve { curve: c }) => {
                    transition_curve = Some(c);
                }
                Some(ParsedDirective::TransitionTime { duration: d }) => {
                    transition_time = Some(d);
                }
                Some(ParsedDirective::TransitionInterval { interval: i }) => {
                    transition_interval = Some(i);
                }
                _ => bail!("Unsupported directive \"{}\"", part),
            },
            Err(e) => bail!("{}", e),
        }
    }

    Ok(MtxtRecord::ControlChange {
        time,
        note,
        controller: controller.to_string(),
        value,
        channel,
        transition_curve,
        transition_time,
        transition_interval,
    })
}

fn parse_voice_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    let mut channel: Option<u16> = None;
    let mut idx = 0;

    // Parse optional channel parameter first
    if let Some(part) = parts.get(idx) {
        let directive = try_parse_directive(part);
        match directive {
            Ok(d) => match d {
                Some(ParsedDirective::Channel { channel: ch }) => {
                    channel = Some(ch);
                    idx += 1;
                }
                None => {}
                _ => bail!("Unsupported directive \"{}\"", part),
            },
            Err(e) => bail!("{}", e),
        }
    }

    let rest = &parts[idx..];
    if rest.is_empty() {
        bail!("Voice event requires voice list");
    }

    let voices = VoiceList::parse(&rest.join(" "));

    Ok(MtxtRecord::Voice {
        time,
        voices,
        channel,
    })
}

fn parse_tuning_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.len() != 2 {
        bail!("Tuning event requires target and cents");
    }

    let target = parts[0].to_string();
    let cents: f32 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid cents value"))?;

    Ok(MtxtRecord::Tuning {
        time,
        target,
        cents,
    })
}

fn parse_reset_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.len() != 1 {
        bail!("Reset event requires target");
    }

    let target = parts[0].to_string();

    Ok(MtxtRecord::Reset { time, target })
}

fn parse_tempo_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.is_empty() {
        bail!("Tempo event requires a BPM value");
    }

    let bpm: f32 = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid BPM value"))?;

    let mut transition_curve = None;
    let mut transition_time = None;
    let mut transition_interval = None;

    for part in &parts[1..] {
        let directive = try_parse_directive(part);
        match directive {
            Ok(d) => {
                if let Some(d) = d {
                    match d {
                        ParsedDirective::TransitionCurve { curve } => {
                            transition_curve = Some(curve)
                        }
                        ParsedDirective::TransitionTime { duration } => {
                            transition_time = Some(duration)
                        }
                        ParsedDirective::TransitionInterval { interval } => {
                            transition_interval = Some(interval)
                        }
                        _ => bail!("Unsupported directive \"{}\"", part),
                    }
                } else {
                    bail!("Invalid tempo command");
                }
            }
            Err(e) => bail!("{}", e),
        }
    }

    Ok(MtxtRecord::Tempo {
        time,
        bpm,
        transition_curve,
        transition_time,
        transition_interval,
    })
}

fn parse_time_signature_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.len() != 1 {
        bail!("Time signature event requires signature");
    }

    let signature: TimeSignature = parts[0].parse().map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(MtxtRecord::TimeSignature { time, signature })
}

fn parse_meta_event(time: Option<BeatTime>, parts: &[&str]) -> Result<MtxtRecord> {
    if parts.is_empty() {
        bail!("Meta event requires type and value");
    }

    if parts[0] == "global" {
        if parts.len() < 3 {
            bail!("Global meta event requires type and value");
        }
        let meta_type = parts[1].to_string();
        let value = parts[2..].join(" ");
        return Ok(MtxtRecord::GlobalMeta { meta_type, value });
    }

    let mut channel = None;
    let mut index = 0;

    // Check for channel directive
    if let Ok(Some(ParsedDirective::Channel { channel: ch })) = try_parse_directive(parts[index]) {
        channel = Some(ch);
        index += 1;
    }

    if parts.len() - index < 2 {
        bail!("Meta event requires type and value");
    }

    let meta_type = parts[index].to_string();
    let value = parts[index + 1..].join(" ");

    Ok(MtxtRecord::Meta {
        time,
        channel,
        meta_type,
        value,
    })
}

fn parse_sysex_event(time: BeatTime, parts: &[&str]) -> Result<MtxtRecord> {
    let mut data = Vec::new();

    for part in parts {
        let byte = u8::from_str_radix(part, 16)
            .map_err(|_| anyhow::anyhow!("Invalid hex byte: {}", part))?;
        data.push(byte);
    }

    Ok(MtxtRecord::SysEx { time, data })
}

fn try_parse_time_event(parts: &[&str]) -> Result<Option<MtxtRecord>> {
    if parts.len() < 2 {
        return Ok(None);
    }

    let time: Result<BeatTime> = parts[0].parse();

    if time.is_err() {
        return Ok(None);
    }

    let time = time.unwrap();

    let res = match parts[1] {
        "note" => parse_note_event(time, &parts[2..]),
        "on" => parse_note_on_event(time, &parts[2..]),
        "off" => parse_note_off_event(time, &parts[2..]),
        "cc" => parse_control_change_event(time, &parts[2..]),
        "voice" => parse_voice_event(time, &parts[2..]),
        "tempo" => parse_tempo_event(time, &parts[2..]),
        "timesig" => parse_time_signature_event(time, &parts[2..]),
        "tuning" => parse_tuning_event(time, &parts[2..]),
        "reset" => parse_reset_event(time, &parts[2..]),
        "meta" => parse_meta_event(Some(time), &parts[2..]),
        "sysex" => parse_sysex_event(time, &parts[2..]),
        _ => bail!("Unknown event type: {}", parts[1]),
    }?;

    Ok(Some(res))
}

pub fn parse_mtxt_line(line: &str) -> Result<MtxtRecord, anyhow::Error> {
    let line = line.trim();

    if line.is_empty() {
        return Ok(MtxtRecord::EmptyLine);
    }

    if line.starts_with("//") {
        let comment_text = line.strip_prefix("//").unwrap_or("").trim().to_string();
        return Ok(MtxtRecord::Comment { text: comment_text });
    }

    let parts: Vec<&str> = line.split_ascii_whitespace().collect();
    if parts.is_empty() {
        return Ok(MtxtRecord::EmptyLine);
    }

    match parts[0] {
        "mtxt" => {
            if parts.len() != 2 {
                bail!(
                    "Invalid file version. Got \"{}\". Expected \"mtxt 1.0\".",
                    parts.join(" ")
                );
            }
            let version: Version = parts[1].parse().map_err(|e| anyhow::anyhow!("{}", e))?;
            version.fail_if_not_supported()?;
            return Ok(MtxtRecord::Header { version });
        }

        "meta" => {
            return parse_meta_event(None, &parts[1..]);
        }

        "alias" => {
            if parts.len() < 3 {
                bail!("alias requires name and at least one note");
            }
            let name = parts[1].to_string();
            if name.parse::<Note>().is_ok() {
                bail!("Cannot redefine note \"{}\" as alias", name);
            }
            let mut notes = Vec::new();
            let merged_notes = parts[2..].join(" ");
            for note_str in merged_notes.split(',') {
                let note: Note = note_str
                    .trim()
                    .parse()
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                notes.push(note);
            }
            let alias_def = Rc::new(AliasDefinition {
                name: name.clone(),
                notes,
            });
            return Ok(MtxtRecord::AliasDef { value: alias_def });
        }
        _ => {}
    };

    let parsed_directive = try_parse_global_directive(parts[0])?;
    if let Some(record) = parsed_directive {
        if parts.len() > 1 {
            bail!("Cannot parse global directive {}", parts.join(" "));
        }
        return Ok(record);
    }

    let parsed_time_event = try_parse_time_event(&parts)?;
    if let Some(record) = parsed_time_event {
        return Ok(record);
    }

    bail!("Cannot parse \"{}\"", parts.join(" "));
}

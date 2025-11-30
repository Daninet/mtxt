use crate::file::MtxtFile;
use crate::types::beat_time::BeatTime;
use crate::types::note::NoteTarget;
use crate::types::record::MtxtRecord;
use crate::types::time_signature::TimeSignature;
use crate::types::version::Version;
use anyhow::Result;
use midly::num::u4;
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::escape::escape_string;
use super::shared::{midi_cc_to_name, midi_key_signature_to_string, midi_key_to_note};

#[derive(Debug)]
enum TickEvent {
    Note {
        start_tick: u32,
        end_tick: u32,
        note: crate::types::note::Note,
        velocity: f32,
        off_velocity: f32,
        channel: u16,
    },
    Other {
        tick: u32,
        record: MtxtRecord,
    },
}

pub fn convert_midi_to_mtxt(path: &str, verbose: bool) -> Result<MtxtFile> {
    let input_path = PathBuf::from(path);

    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {}", path);
    }

    if verbose {
        println!("Reading MIDI file: {}", input_path.display());
    }

    let data = fs::read(&input_path)?;
    let smf = Smf::parse(&data)?;

    if verbose {
        println!("Converting MIDI to MTXT...");
    }

    let mtxt_file = convert_smf_to_mtxt(&smf)?;

    if verbose {
        println!("Conversion complete: {} records", mtxt_file.records.len());
    }

    Ok(mtxt_file)
}

fn convert_smf_to_mtxt(smf: &Smf) -> Result<MtxtFile> {
    let mut mtxt_file = MtxtFile::new();
    mtxt_file.records.push(MtxtRecord::Header {
        version: Version { major: 1, minor: 0 },
    });

    // Get timing information
    let ticks_per_quarter: u16 = match smf.header.timing {
        Timing::Metrical(ticks) => ticks.as_int(),
        Timing::Timecode(_, _) => 480, // Default fallback
    };

    // Collect all events from all tracks with their tick times
    // For notes, we need to track both start and end ticks
    let mut all_events: Vec<TickEvent> = Vec::new();

    // Convert each track
    for (_track_idx, track) in smf.tracks.iter().enumerate() {
        let mut current_time_ticks = 0u32;
        let mut note_on_events: HashMap<(u8, u8), (u32, f32)> = HashMap::new(); // (channel, key) -> (tick_time, velocity)

        // Heuristic: associate track with a channel (Type 1 MIDI)
        // If we are in a multi-track file, tracks often correspond to a single channel.
        // We scan the track for the first channel event to determine the "track channel".
        let mut track_channel: Option<u8> = None;
        if smf.header.format != Format::SingleTrack {
            for event in track.iter() {
                if let TrackEventKind::Midi { channel, .. } = event.kind {
                    track_channel = Some(channel.as_int());
                    break;
                }
            }
        }

        for event in track.iter() {
            current_time_ticks += event.delta.as_int();

            match &event.kind {
                TrackEventKind::Midi { channel, message } => {
                    convert_midi_message_to_tick_events(
                        message,
                        *channel,
                        &mut note_on_events,
                        current_time_ticks,
                        &mut all_events,
                    )?;
                }
                TrackEventKind::Meta(meta_msg) => {
                    if let Some(record) = convert_meta_message(
                        meta_msg,
                        current_time_ticks,
                        _track_idx == 0,
                        track_channel,
                    )? {
                        all_events.push(TickEvent::Other {
                            tick: current_time_ticks,
                            record,
                        });
                    }
                }
                TrackEventKind::SysEx(data) => {
                    all_events.push(TickEvent::Other {
                        tick: current_time_ticks,
                        record: MtxtRecord::SysEx {
                            time: BeatTime::zero(), // Will be set later
                            data: data.to_vec(),
                        },
                    });
                }
                TrackEventKind::Escape(_data) => {
                    // Escape events are rare and can be skipped
                }
            }
        }

        // Handle any remaining note-on events without corresponding note-off
        for ((channel, key), (tick_time, velocity)) in note_on_events {
            if let Ok(note) = midi_key_to_note(key) {
                all_events.push(TickEvent::Note {
                    start_tick: tick_time,
                    end_tick: tick_time + (ticks_per_quarter as u32), // Default 1 beat
                    note,
                    velocity,
                    off_velocity: 0.0,
                    channel: channel as u16,
                });
            }
        }
    }

    // Sort all events by their primary tick time (start_tick for notes, tick for others)
    all_events.sort_by_key(|event| match event {
        TickEvent::Note { start_tick, .. } => *start_tick,
        TickEvent::Other { tick, .. } => *tick,
    });

    // Convert tick times to beat times, accounting for tempo changes
    let mut tick_to_beat_map: HashMap<u32, BeatTime> = HashMap::new();

    // First, collect all unique tick times we need to convert
    let mut all_ticks: Vec<u32> = Vec::new();
    for event in &all_events {
        match event {
            TickEvent::Note {
                start_tick,
                end_tick,
                ..
            } => {
                all_ticks.push(*start_tick);
                all_ticks.push(*end_tick);
            }
            TickEvent::Other { tick, .. } => {
                all_ticks.push(*tick);
            }
        }
    }
    all_ticks.sort();
    all_ticks.dedup();

    // Convert all tick times to beat times, tracking tempo changes
    let mut current_tick = 0u32;
    let mut current_beat = 0.0f64;
    tick_to_beat_map.insert(0, BeatTime::zero());

    for &tick in &all_ticks {
        if tick == 0 {
            continue;
        }

        let tick_delta = tick - current_tick;
        if tick_delta > 0 {
            let beat_delta = tick_delta as f64 / ticks_per_quarter as f64;
            current_beat += beat_delta;
        }
        current_tick = tick;

        let whole_beats = current_beat.floor() as u32;
        let frac_beats = (current_beat - whole_beats as f64) as f32;
        tick_to_beat_map.insert(tick, BeatTime::from_parts(whole_beats, frac_beats));
    }

    // Now convert all events to MtxtRecords with proper beat times
    let mut final_events: Vec<MtxtRecord> = Vec::new();

    for event in all_events {
        match event {
            TickEvent::Note {
                start_tick,
                end_tick,
                note,
                velocity,
                off_velocity,
                channel,
            } => {
                let start_beat = *tick_to_beat_map
                    .get(&start_tick)
                    .unwrap_or(&BeatTime::zero());
                let end_beat = *tick_to_beat_map.get(&end_tick).unwrap_or(&start_beat);
                let duration = end_beat - start_beat;

                final_events.push(MtxtRecord::Note {
                    time: start_beat,
                    note: NoteTarget::Note(note),
                    duration: Some(duration),
                    velocity: Some(velocity),
                    off_velocity: Some(off_velocity),
                    channel: Some(channel),
                });
            }
            TickEvent::Other { tick, mut record } => {
                let beat_time = *tick_to_beat_map.get(&tick).unwrap_or(&BeatTime::zero());

                // Update the record's time
                match &mut record {
                    MtxtRecord::Tempo { time, .. } => {
                        *time = beat_time;
                    }
                    MtxtRecord::ControlChange { time, .. }
                    | MtxtRecord::TimeSignature { time, .. }
                    | MtxtRecord::Voice { time, .. }
                    | MtxtRecord::SysEx { time, .. } => {
                        *time = beat_time;
                    }
                    MtxtRecord::Meta { time, .. } => {
                        if beat_time == BeatTime::zero() {
                            *time = None;
                        } else {
                            *time = Some(beat_time);
                        }
                    }
                    _ => {}
                }

                final_events.push(record);
            }
        }
    }

    // Sort final events to ensure None/GlobalMeta come first
    final_events.sort_by(|a, b| {
        // Helper to get sort key: (order_group, time)
        // order_group: 0=GlobalMeta, 1=Meta(None), 2=Other
        fn get_sort_key(record: &MtxtRecord) -> (u8, BeatTime) {
            match record {
                MtxtRecord::GlobalMeta { .. } => (0, BeatTime::zero()),
                MtxtRecord::Meta { time: None, .. } => (1, BeatTime::zero()),
                MtxtRecord::Header { .. } => (0, BeatTime::zero()), // Should not be here but handled for completeness
                // For records with time, use that time
                MtxtRecord::Meta { time: Some(t), .. } => (2, *t),
                MtxtRecord::Note { time, .. }
                | MtxtRecord::NoteOn { time, .. }
                | MtxtRecord::NoteOff { time, .. }
                | MtxtRecord::ControlChange { time, .. }
                | MtxtRecord::Voice { time, .. }
                | MtxtRecord::Tempo { time, .. }
                | MtxtRecord::TimeSignature { time, .. }
                | MtxtRecord::SysEx { time, .. } => (2, *time),
                // Directives usually don't have time, treat as time 0 or context dependent
                // But in this converter we produce timed records mostly.
                // Assuming other records have effective time 0 if not specified
                _ => (2, BeatTime::zero()),
            }
        }

        let (group_a, time_a) = get_sort_key(a);
        let (group_b, time_b) = get_sort_key(b);

        if group_a != group_b {
            group_a.cmp(&group_b)
        } else {
            time_a.cmp(&time_b)
        }
    });

    // Add events to the file
    mtxt_file.records.extend(final_events);

    Ok(mtxt_file)
}

fn convert_midi_message_to_tick_events(
    msg: &MidiMessage,
    channel: u4,
    note_on_events: &mut HashMap<(u8, u8), (u32, f32)>,
    current_tick: u32,
    tick_events: &mut Vec<TickEvent>,
) -> Result<()> {
    let channel_u8 = channel.as_int();

    match msg {
        MidiMessage::NoteOn { key, vel } => {
            let velocity = vel.as_int() as f32 / 127.0;
            if velocity > 0.0 {
                // Store note-on event with tick time
                note_on_events.insert((channel_u8, key.as_int()), (current_tick, velocity));
            } else {
                // Velocity 0 note-on is treated as note-off
                if let Some((start_tick, note_velocity)) =
                    note_on_events.remove(&(channel_u8, key.as_int()))
                {
                    let note = midi_key_to_note(key.as_int())?;
                    tick_events.push(TickEvent::Note {
                        start_tick,
                        end_tick: current_tick,
                        note,
                        velocity: note_velocity,
                        off_velocity: 0.0,
                        channel: channel_u8 as u16,
                    });
                }
            }
        }
        MidiMessage::NoteOff { key, vel } => {
            if let Some((start_tick, note_velocity)) =
                note_on_events.remove(&(channel_u8, key.as_int()))
            {
                let note = midi_key_to_note(key.as_int())?;
                tick_events.push(TickEvent::Note {
                    start_tick,
                    end_tick: current_tick,
                    note,
                    velocity: note_velocity,
                    off_velocity: vel.as_int() as f32 / 127.0,
                    channel: channel_u8 as u16,
                });
            }
        }
        MidiMessage::Controller { controller, value } => {
            let controller_name = midi_cc_to_name(controller.as_int());
            let mtxt_value = value.as_int() as f32 / 127.0;

            tick_events.push(TickEvent::Other {
                tick: current_tick,
                record: MtxtRecord::ControlChange {
                    time: BeatTime::zero(),
                    note: None,
                    controller: controller_name,
                    value: mtxt_value,
                    channel: Some(channel_u8 as u16),
                    transition_curve: None,
                    transition_time: None,
                    transition_interval: None,
                },
            });
        }
        MidiMessage::ProgramChange { program } => {
            tick_events.push(TickEvent::Other {
                tick: current_tick,
                record: MtxtRecord::Voice {
                    time: BeatTime::zero(),
                    voices: vec![program.as_int().to_string()],
                    channel: Some(channel_u8 as u16),
                },
            });
        }
        MidiMessage::PitchBend { bend } => {
            let bend_value = (bend.as_int() as f32 - 8192.0) / 8192.0 * 12.0;

            tick_events.push(TickEvent::Other {
                tick: current_tick,
                record: MtxtRecord::ControlChange {
                    time: BeatTime::zero(),
                    note: None,
                    controller: "pitch".to_string(),
                    value: bend_value,
                    channel: Some(channel_u8 as u16),
                    transition_curve: None,
                    transition_time: None,
                    transition_interval: None,
                },
            });
        }
        MidiMessage::Aftertouch { key: _, vel } | MidiMessage::ChannelAftertouch { vel } => {
            let value = vel.as_int() as f32 / 127.0;
            tick_events.push(TickEvent::Other {
                tick: current_tick,
                record: MtxtRecord::ControlChange {
                    time: BeatTime::zero(),
                    note: None,
                    controller: "aftertouch".to_string(),
                    value,
                    channel: Some(channel_u8 as u16),
                    transition_curve: None,
                    transition_time: None,
                    transition_interval: None,
                },
            });
        }
    }

    Ok(())
}

fn convert_meta_message(
    msg: &MetaMessage,
    current_tick: u32,
    is_first_track: bool,
    track_channel: Option<u8>,
) -> Result<Option<MtxtRecord>> {
    match msg {
        MetaMessage::Tempo(tempo) => {
            let tempo_us = tempo.as_int() as f32;
            let bpm = 60_000_000.0 / tempo_us;
            Ok(Some(MtxtRecord::Tempo {
                time: BeatTime::zero(),
                bpm,
                transition_curve: None,
                transition_time: None,
                transition_interval: None,
            }))
        }
        MetaMessage::TimeSignature(num, den, _clocks, _bb) => {
            let signature = TimeSignature {
                numerator: *num,
                denominator: 1 << den,
            };
            Ok(Some(MtxtRecord::TimeSignature {
                time: BeatTime::zero(),
                signature,
            }))
        }
        MetaMessage::TrackName(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            if track_channel.is_none() {
                if is_first_track {
                    Ok(Some(MtxtRecord::GlobalMeta {
                        meta_type: "title".to_string(),
                        value,
                    }))
                } else {
                    Ok(Some(MtxtRecord::GlobalMeta {
                        meta_type: "text".to_string(),
                        value,
                    }))
                }
            } else {
                Ok(Some(MtxtRecord::Meta {
                    time: Some(BeatTime::zero()),
                    channel: track_channel.map(|c| c as u16),
                    meta_type: "name".to_string(),
                    value,
                }))
            }
        }
        MetaMessage::Text(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            if track_channel.is_none() {
                Ok(Some(MtxtRecord::GlobalMeta {
                    meta_type: "text".to_string(),
                    value,
                }))
            } else {
                Ok(Some(MtxtRecord::Meta {
                    time: Some(BeatTime::zero()),
                    channel: track_channel.map(|c| c as u16),
                    meta_type: "text".to_string(),
                    value,
                }))
            }
        }
        MetaMessage::Copyright(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::GlobalMeta {
                meta_type: "copyright".to_string(),
                value,
            }))
        }
        MetaMessage::InstrumentName(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(BeatTime::zero()),
                channel: track_channel.map(|c| c as u16),
                meta_type: "instrument".to_string(),
                value,
            }))
        }
        MetaMessage::Lyric(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(BeatTime::zero()),
                channel: track_channel.map(|c| c as u16),
                meta_type: "lyric".to_string(),
                value,
            }))
        }
        MetaMessage::Marker(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(BeatTime::zero()),
                channel: track_channel.map(|c| c as u16),
                meta_type: "marker".to_string(),
                value,
            }))
        }
        MetaMessage::CuePoint(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(BeatTime::zero()),
                channel: track_channel.map(|c| c as u16),
                meta_type: "cue".to_string(),
                value,
            }))
        }
        MetaMessage::ProgramName(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::GlobalMeta {
                meta_type: "program".to_string(),
                value,
            }))
        }
        MetaMessage::DeviceName(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::GlobalMeta {
                meta_type: "device".to_string(),
                value,
            }))
        }
        MetaMessage::TrackNumber(track_num) => {
            if let Some(num) = track_num {
                Ok(Some(MtxtRecord::Meta {
                    time: Some(BeatTime::zero()),
                    channel: None,
                    meta_type: "tracknumber".to_string(),
                    value: num.to_string(),
                }))
            } else {
                Ok(None)
            }
        }
        MetaMessage::MidiChannel(channel) => Ok(Some(MtxtRecord::Meta {
            time: Some(BeatTime::zero()),
            channel: None,
            meta_type: "midichannel".to_string(),
            value: channel.as_int().to_string(),
        })),
        MetaMessage::MidiPort(port) => Ok(Some(MtxtRecord::Meta {
            time: Some(BeatTime::zero()),
            channel: None,
            meta_type: "midiport".to_string(),
            value: port.as_int().to_string(),
        })),
        MetaMessage::SmpteOffset(smpte) => {
            let value = format!("{:?}", smpte);
            Ok(Some(MtxtRecord::GlobalMeta {
                meta_type: "smpte".to_string(),
                value,
            }))
        }
        MetaMessage::KeySignature(sharps_flats, minor) => {
            let value = midi_key_signature_to_string(*sharps_flats, *minor);

            // If it's at the beginning, treat as global Key
            if current_tick == 0 {
                Ok(Some(MtxtRecord::GlobalMeta {
                    meta_type: "key".to_string(),
                    value,
                }))
            } else {
                // Otherwise treat as timed KeySignature
                Ok(Some(MtxtRecord::Meta {
                    time: Some(BeatTime::zero()),
                    channel: None,
                    meta_type: "keysignature".to_string(),
                    value,
                }))
            }
        }
        MetaMessage::SequencerSpecific(data) => {
            let hex_str = data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join("");

            Ok(Some(MtxtRecord::Meta {
                time: Some(BeatTime::zero()),
                channel: None,
                meta_type: "sequencerspecific".to_string(),
                value: hex_str,
            }))
        }
        MetaMessage::Unknown(msg_type, data) => {
            let hex_str = data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join("");

            Ok(Some(MtxtRecord::Meta {
                time: Some(BeatTime::zero()),
                channel: None,
                meta_type: format!("unknown_{:02X}", msg_type),
                value: hex_str,
            }))
        }
        MetaMessage::EndOfTrack => Ok(None),
    }
}

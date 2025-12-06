use crate::file::MtxtFile;
use crate::types::beat_time::BeatTime;
use crate::types::note::NoteTarget;
use crate::types::record::MtxtRecord;
use crate::types::time_signature::TimeSignature;
use crate::types::version::Version;
use anyhow::{Result, bail};
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::fs;
use std::path::PathBuf;

use super::escape::escape_string;
use super::shared::{midi_cc_to_name, midi_key_signature_to_string, midi_key_to_note};

#[derive(Debug)]
struct MidiSingleTrackEvent {
    tick: BeatTime,
    record: MtxtRecord,
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

// It merges all events from all MIDI tracks into a single list of events
fn get_midi_single_track_events(smf: &Smf) -> Result<Vec<MidiSingleTrackEvent>> {
    let mut all_events: Vec<MidiSingleTrackEvent> = Vec::new();

    // MIDI format 0 is a single track file
    // MIDI format 1 is a synchronous multi-track file (first track usually is the tempo track)
    // MIDI format 2 is an asynchronous multi-track file (each track has its own timing, no common time signature)

    if smf.header.format == Format::Sequential {
        bail!("MIDI format 2 files are not yet supported");
    }

    let ppqn = match smf.header.timing {
        Timing::Metrical(ppqn) => ppqn.as_int() as u64,
        Timing::Timecode(_, _) => bail!("Timecode timing is not yet supported"),
    };

    for (_track_idx, track) in smf.tracks.iter().enumerate() {
        let mut current_raw_ticks = 0u64;

        // Heuristic: associate track with a channel (Type 1 MIDI)
        let mut guessed_track_channel: Option<u8> = None;
        if smf.header.format != Format::SingleTrack {
            for event in track.iter() {
                if let TrackEventKind::Midi { channel, .. } = event.kind {
                    guessed_track_channel = Some(channel.as_int());
                    break;
                }
            }
        }

        for event in track.iter() {
            current_raw_ticks += event.delta.as_int() as u64;
            let whole_parts = current_raw_ticks / ppqn;
            let frac_parts = current_raw_ticks % ppqn;
            let beat_time =
                BeatTime::from_parts(whole_parts as u32, frac_parts as f32 / ppqn as f32);

            match &event.kind {
                TrackEventKind::Midi { channel, message } => {
                    let record = convert_midi_message_to_record(
                        message,
                        channel.as_int() as u16,
                        beat_time,
                    )?;
                    all_events.push(MidiSingleTrackEvent {
                        tick: beat_time,
                        record,
                    });
                }
                TrackEventKind::Meta(meta_msg) => {
                    if let Some(record) = convert_meta_message(
                        meta_msg,
                        beat_time,
                        _track_idx == 0,
                        guessed_track_channel,
                    )? {
                        all_events.push(MidiSingleTrackEvent {
                            tick: beat_time,
                            record,
                        });
                    }
                }
                TrackEventKind::SysEx(data) => {
                    all_events.push(MidiSingleTrackEvent {
                        tick: beat_time,
                        record: MtxtRecord::SysEx {
                            time: beat_time,
                            data: data.to_vec(),
                        },
                    });
                }
                TrackEventKind::Escape(data) => {
                    let formatted: String =
                        data.iter().map(|byte| format!(" {:02x}", byte)).collect();

                    all_events.push(MidiSingleTrackEvent {
                        tick: beat_time,
                        record: MtxtRecord::Comment {
                            text: format!("Escape sequence: {}", formatted.trim()),
                        },
                    });
                }
            }
        }
    }

    all_events.sort_by_key(|event| event.tick);
    Ok(all_events)
}

fn convert_smf_to_mtxt(smf: &Smf) -> Result<MtxtFile> {
    let mut mtxt_file = MtxtFile::new();
    mtxt_file.records.push(MtxtRecord::Header {
        version: Version { major: 1, minor: 0 },
    });

    let all_events = get_midi_single_track_events(smf)?;
    let mut final_events: Vec<MtxtRecord> = Vec::new();

    for event in all_events {
        let record = event.record.clone();
        final_events.push(record);
    }

    // Sort final events to ensure None/GlobalMeta come first
    final_events.sort_by(|a, b| {
        // Helper to get sort key: (order_group, time)
        // order_group: 0=GlobalMeta, 1=Meta(None), 2=Other
        fn get_sort_key(record: &MtxtRecord) -> (u8, BeatTime) {
            match record {
                MtxtRecord::GlobalMeta { .. } => (0, BeatTime::zero()),
                MtxtRecord::Meta { time: None, .. } => (1, BeatTime::zero()),
                MtxtRecord::Header { .. } => (0, BeatTime::zero()),
                MtxtRecord::Meta { time: Some(t), .. } => (2, *t),
                MtxtRecord::Note { time, .. }
                | MtxtRecord::NoteOn { time, .. }
                | MtxtRecord::NoteOff { time, .. }
                | MtxtRecord::ControlChange { time, .. }
                | MtxtRecord::Voice { time, .. }
                | MtxtRecord::Tempo { time, .. }
                | MtxtRecord::TimeSignature { time, .. }
                | MtxtRecord::SysEx { time, .. } => (2, *time),
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

    mtxt_file.records.extend(final_events);

    Ok(mtxt_file)
}

fn convert_midi_message_to_record(
    msg: &MidiMessage,
    channel: u16,
    beat_time: BeatTime,
) -> Result<MtxtRecord> {
    match msg {
        MidiMessage::NoteOn { key, vel } => {
            let note = midi_key_to_note(key.as_int())?;
            let velocity = vel.as_int() as f32 / 127.0;
            return Ok(MtxtRecord::NoteOn {
                time: beat_time,
                note: NoteTarget::Note(note),
                velocity: Some(velocity),
                channel: Some(channel),
            });
        }
        MidiMessage::NoteOff { key, vel } => {
            let note = midi_key_to_note(key.as_int())?;
            let off_velocity = vel.as_int() as f32 / 127.0;

            return Ok(MtxtRecord::NoteOff {
                time: beat_time,
                note: NoteTarget::Note(note),
                off_velocity: Some(off_velocity),
                channel: Some(channel),
            });
        }
        MidiMessage::Controller { controller, value } => {
            let controller_name = midi_cc_to_name(controller.as_int());
            let mtxt_value = value.as_int() as f32 / 127.0;

            return Ok(MtxtRecord::ControlChange {
                time: beat_time,
                note: None,
                controller: controller_name,
                value: mtxt_value,
                channel: Some(channel),
                transition_curve: None,
                transition_time: None,
                transition_interval: None,
            });
        }
        MidiMessage::ProgramChange { program } => {
            return Ok(MtxtRecord::Voice {
                time: beat_time,
                voices: vec![program.as_int().to_string()],
                channel: Some(channel),
            });
        }
        MidiMessage::PitchBend { bend } => {
            let bend_value = (bend.as_int() as f32 - 8192.0) / 8192.0 * 12.0;

            return Ok(MtxtRecord::ControlChange {
                time: beat_time,
                note: None,
                controller: "pitch".to_string(),
                value: bend_value,
                channel: Some(channel),
                transition_curve: None,
                transition_time: None,
                transition_interval: None,
            });
        }
        MidiMessage::Aftertouch { key: _, vel } | MidiMessage::ChannelAftertouch { vel } => {
            let value = vel.as_int() as f32 / 127.0;
            return Ok(MtxtRecord::ControlChange {
                time: beat_time,
                note: None,
                controller: "aftertouch".to_string(),
                value,
                channel: Some(channel),
                transition_curve: None,
                transition_time: None,
                transition_interval: None,
            });
        }
    }
}

fn convert_meta_message(
    msg: &MetaMessage,
    beat_time: BeatTime,
    is_first_track: bool,
    track_channel: Option<u8>,
) -> Result<Option<MtxtRecord>> {
    match msg {
        MetaMessage::Tempo(tempo) => {
            let tempo_us = tempo.as_int() as f32;
            let bpm = 60_000_000.0 / tempo_us;
            Ok(Some(MtxtRecord::Tempo {
                time: beat_time,
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
                time: beat_time,
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
                    time: Some(beat_time),
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
                    time: Some(beat_time),
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
                time: Some(beat_time),
                channel: track_channel.map(|c| c as u16),
                meta_type: "instrument".to_string(),
                value,
            }))
        }
        MetaMessage::Lyric(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(beat_time),
                channel: track_channel.map(|c| c as u16),
                meta_type: "lyric".to_string(),
                value,
            }))
        }
        MetaMessage::Marker(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(beat_time),
                channel: track_channel.map(|c| c as u16),
                meta_type: "marker".to_string(),
                value,
            }))
        }
        MetaMessage::CuePoint(text) => {
            let value = escape_string(&String::from_utf8_lossy(text));
            Ok(Some(MtxtRecord::Meta {
                time: Some(beat_time),
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
                    time: Some(beat_time),
                    channel: None,
                    meta_type: "tracknumber".to_string(),
                    value: num.to_string(),
                }))
            } else {
                Ok(None)
            }
        }
        MetaMessage::MidiChannel(channel) => Ok(Some(MtxtRecord::Meta {
            time: Some(beat_time),
            channel: None,
            meta_type: "midichannel".to_string(),
            value: channel.as_int().to_string(),
        })),
        MetaMessage::MidiPort(port) => Ok(Some(MtxtRecord::Meta {
            time: Some(beat_time),
            channel: None,
            meta_type: "midiport".to_string(),
            value: port.as_int().to_string(),
        })),
        MetaMessage::SmpteOffset(smpte) => {
            // HH:MM:SS:FF (Hours:Minutes:Seconds:Frames)
            let value = format!(
                "{:02}:{:02}:{:02}:{:02}",
                smpte.hour(),
                smpte.minute(),
                smpte.second(),
                smpte.frame()
            );

            Ok(Some(MtxtRecord::GlobalMeta {
                meta_type: "smpte".to_string(),
                value,
            }))
        }
        MetaMessage::KeySignature(sharps_flats, minor) => {
            let value = midi_key_signature_to_string(*sharps_flats, *minor);

            // If it's at the beginning, treat as global Key
            if beat_time == BeatTime::zero() {
                Ok(Some(MtxtRecord::GlobalMeta {
                    meta_type: "key".to_string(),
                    value,
                }))
            } else {
                // Otherwise treat as timed KeySignature
                Ok(Some(MtxtRecord::Meta {
                    time: Some(beat_time),
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
                time: Some(beat_time),
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
                time: Some(beat_time),
                channel: None,
                meta_type: format!("unknown_{:02X}", msg_type),
                value: hex_str,
            }))
        }
        MetaMessage::EndOfTrack => Ok(None),
    }
}

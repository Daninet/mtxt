use crate::file::MtxtFile;
use crate::types::output_record::MtxtOutputRecord;
use anyhow::Result;
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::path::PathBuf;

use super::escape::unescape_string;
use super::shared::{
    MidiControllerEvent, controller_name_to_midi, note_to_midi_number, time_signature_to_midi,
};

pub fn convert_mtxt_to_midi(mtxt_file: &MtxtFile, output: &str, verbose: bool) -> Result<()> {
    let output_path = PathBuf::from(output);

    if verbose {
        println!("Converting to MIDI...");
    }

    let mut output_records = mtxt_file.get_output_records();

    if verbose {
        println!("Processing {} output records", output_records.len());
    }

    let smf = convert_output_records_to_midi(&mut output_records)?;

    if verbose {
        println!("Writing MIDI file: {}", output_path.display());
    }

    smf.save(&output_path)?;

    if verbose {
        println!("Conversion completed successfully!");
    }

    Ok(())
}

fn convert_output_records_to_midi(records: &mut [MtxtOutputRecord]) -> Result<Smf<'_>> {
    // Use 480 ticks per quarter note (standard resolution)
    let ticks_per_beat = 480;
    let timing = Timing::Metrical(midly::num::u15::new(ticks_per_beat));

    // Create a single track (Format 0)
    let mut track_events = Vec::new();

    // Track current tempo to convert microseconds to beats
    let mut current_bpm = 120.0; // Default tempo

    // Process each output record
    for record in records.iter_mut() {
        // Convert microseconds to beats using current BPM, then beats to ticks
        let time_micros = record.time();
        let tick = micros_to_ticks(time_micros, current_bpm, ticks_per_beat);

        // Update current_bpm if this is a tempo change
        if let MtxtOutputRecord::Tempo { bpm, .. } = record {
            current_bpm = *bpm as f64;
        }

        match record {
            MtxtOutputRecord::NoteOn {
                note,
                velocity,
                channel,
                ..
            } => {
                let note_num = note_to_midi_number(note)?;
                let vel = (*velocity * 127.0).clamp(0.0, 127.0) as u8;
                let ch = (*channel as u8).min(15);

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(tick),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::new(ch),
                        message: MidiMessage::NoteOn {
                            key: midly::num::u7::new(note_num),
                            vel: midly::num::u7::new(vel),
                        },
                    },
                });
            }
            MtxtOutputRecord::NoteOff {
                note,
                off_velocity,
                channel,
                ..
            } => {
                let note_num = note_to_midi_number(note)?;
                let vel = (*off_velocity * 127.0).clamp(0.0, 127.0) as u8;
                let ch = (*channel as u8).min(15);

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(tick),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::new(ch),
                        message: MidiMessage::NoteOff {
                            key: midly::num::u7::new(note_num),
                            vel: midly::num::u7::new(vel),
                        },
                    },
                });
            }
            MtxtOutputRecord::ControlChange {
                controller,
                value,
                channel,
                ..
            } => {
                let ch = (*channel as u8).min(15);

                // Convert controller name to MIDI CC number or pitch bend
                match controller_name_to_midi(controller, *value)? {
                    MidiControllerEvent::CC { number, value } => {
                        track_events.push(TrackEvent {
                            delta: midly::num::u28::new(tick),
                            kind: TrackEventKind::Midi {
                                channel: midly::num::u4::new(ch),
                                message: MidiMessage::Controller {
                                    controller: midly::num::u7::new(number),
                                    value: midly::num::u7::new(value),
                                },
                            },
                        });
                    }
                    MidiControllerEvent::PitchBend { value } => {
                        track_events.push(TrackEvent {
                            delta: midly::num::u28::new(tick),
                            kind: TrackEventKind::Midi {
                                channel: midly::num::u4::new(ch),
                                message: MidiMessage::PitchBend {
                                    bend: midly::PitchBend(midly::num::u14::new(value)),
                                },
                            },
                        });
                    }
                    MidiControllerEvent::Aftertouch { value } => {
                        track_events.push(TrackEvent {
                            delta: midly::num::u28::new(tick),
                            kind: TrackEventKind::Midi {
                                channel: midly::num::u4::new(ch),
                                message: MidiMessage::ChannelAftertouch {
                                    vel: midly::num::u7::new(value),
                                },
                            },
                        });
                    }
                }
            }
            MtxtOutputRecord::Voice {
                voices, channel, ..
            } => {
                for voice in voices.iter_mut() {
                    *voice = unescape_string(voice);
                }

                // For now, just use the first voice as a program change if it's a number
                // In a more sophisticated implementation, we'd have a voice-to-program mapping
                if let Some(first_voice) = voices.first() {
                    // Try to parse as a number, otherwise default to 0 (Acoustic Grand Piano)
                    let program = first_voice.parse::<u8>().unwrap_or(0).min(127);
                    let ch = (*channel as u8).min(15);

                    track_events.push(TrackEvent {
                        delta: midly::num::u28::new(tick),
                        kind: TrackEventKind::Midi {
                            channel: midly::num::u4::new(ch),
                            message: MidiMessage::ProgramChange {
                                program: midly::num::u7::new(program),
                            },
                        },
                    });
                }
            }
            MtxtOutputRecord::Tempo { bpm, .. } => {
                // Convert BPM to microseconds per quarter note
                let microseconds_per_quarter = (60_000_000.0 / *bpm) as u32;

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(tick),
                    kind: TrackEventKind::Meta(MetaMessage::Tempo(midly::num::u24::new(
                        microseconds_per_quarter,
                    ))),
                });
            }
            MtxtOutputRecord::TimeSignature { signature, .. } => {
                let (numerator, denominator) = time_signature_to_midi(signature);

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(tick),
                    kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                        numerator,
                        denominator,
                        24, // MIDI clocks per metronome click
                        8,  // 32nd notes per quarter note
                    )),
                });
            }
            MtxtOutputRecord::Reset { .. } => {
                // Reset events don't have a direct MIDI equivalent
                // Could send All Notes Off (CC 123) or All Sound Off (CC 120)
                // For now, just skip it
            }
            MtxtOutputRecord::GlobalMeta {
                meta_type, value, ..
            }
            | MtxtOutputRecord::ChannelMeta {
                meta_type, value, ..
            } => {
                *value = unescape_string(value);
                let meta_bytes = value.as_bytes();
                let kind = match meta_type.as_str() {
                    "copyright" => MetaMessage::Copyright(meta_bytes),
                    "title" | "trackname" | "name" => MetaMessage::TrackName(meta_bytes),
                    "instrument" => MetaMessage::InstrumentName(meta_bytes),
                    "lyric" => MetaMessage::Lyric(meta_bytes),
                    "marker" => MetaMessage::Marker(meta_bytes),
                    "cue" => MetaMessage::CuePoint(meta_bytes),
                    "program" => MetaMessage::ProgramName(meta_bytes),
                    "device" => MetaMessage::DeviceName(meta_bytes),
                    _ => MetaMessage::Text(meta_bytes),
                };

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(tick),
                    kind: TrackEventKind::Meta(kind),
                });
            }
            MtxtOutputRecord::Beat { .. } => {}
            MtxtOutputRecord::SysEx { data, .. } => {
                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(tick),
                    kind: TrackEventKind::SysEx(data),
                });
            }
        }
    }

    // Sort track by tick time
    track_events.sort_by_key(|event| event.delta.as_int());

    // Convert absolute timing to relative (delta) timing
    convert_to_delta_timing(&mut track_events);

    // Add end of track event
    track_events.push(TrackEvent {
        delta: midly::num::u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Ok(Smf {
        header: midly::Header {
            format: midly::Format::SingleTrack,
            timing,
        },
        tracks: vec![track_events],
    })
}

fn micros_to_ticks(time_micros: u64, bpm: f64, ticks_per_beat: u16) -> u32 {
    // Convert microseconds to beats using BPM
    // beats = time_micros / microseconds_per_beat
    // where microseconds_per_beat = 60_000_000 / bpm
    let micros_per_beat = 60_000_000.0 / bpm;
    let beats = time_micros as f64 / micros_per_beat;

    // Convert beats to ticks
    // ticks = beats * ticks_per_beat
    (beats * ticks_per_beat as f64) as u32
}

fn convert_to_delta_timing(events: &mut [TrackEvent]) {
    let mut last_tick = 0u32;
    for event in events.iter_mut() {
        let current_tick = event.delta.as_int();
        event.delta = midly::num::u28::new(current_tick.saturating_sub(last_tick));
        last_tick = current_tick;
    }
}

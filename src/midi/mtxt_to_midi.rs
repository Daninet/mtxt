use crate::file::MtxtFile;
use crate::types::output_record::MtxtOutputRecord;
use anyhow::{Result, bail};
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
    let ppqn = 480;
    let timing = Timing::Metrical(midly::num::u15::new(ppqn));

    let mut track_events = Vec::new();

    let mut current_bpm = 120.0;

    let mut last_micros = 0u64;

    for record in records.iter_mut() {
        let time_micros = record.time();
        assert!(time_micros >= last_micros);
        let delta_micros = time_micros - last_micros;
        last_micros = time_micros;

        let micros_per_beat = 60_000_000.0 / current_bpm;
        let delta_beats = delta_micros as f64 / micros_per_beat;
        let mut delta_tick = (delta_beats * ppqn as f64).round() as u64;

        while delta_tick > midly::num::u28::max_value().as_int() as u64 {
            track_events.push(TrackEvent {
                delta: midly::num::u28::max_value(),
                kind: TrackEventKind::Meta(MetaMessage::Text(b"long delta")),
            });
            delta_tick -= midly::num::u28::max_value().as_int() as u64;
        }

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
                let vel = (*velocity * 127.0) as u8;
                if *channel > 15 {
                    bail!("Channel {} out of range for MIDI", *channel);
                }
                let ch = *channel as u8;

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(delta_tick as u32),
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
                let vel = (*off_velocity * 127.0) as u8;
                if *channel > 15 {
                    bail!("Channel {} out of range for MIDI", *channel);
                }
                let ch = *channel as u8;

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(delta_tick as u32),
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
                if *channel > 15 {
                    bail!("Channel {} out of range for MIDI", *channel);
                }
                let ch = *channel as u8;

                // Convert controller name to MIDI CC number or pitch bend
                match controller_name_to_midi(controller, *value)? {
                    MidiControllerEvent::CC { number, value } => {
                        track_events.push(TrackEvent {
                            delta: midly::num::u28::new(delta_tick as u32),
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
                            delta: midly::num::u28::new(delta_tick as u32),
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
                            delta: midly::num::u28::new(delta_tick as u32),
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
                    let program = first_voice.parse::<u8>().unwrap_or(0);
                    if program > 127 {
                        bail!("Program number out of range for MIDI");
                    }
                    if *channel > 15 {
                        bail!("Channel {} out of range for MIDI", *channel);
                    }
                    let ch = *channel as u8;

                    track_events.push(TrackEvent {
                        delta: midly::num::u28::new(delta_tick as u32),
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
                let microseconds_per_quarter = (60_000_000.0 / *bpm) as u32;

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(delta_tick as u32),
                    kind: TrackEventKind::Meta(MetaMessage::Tempo(midly::num::u24::new(
                        microseconds_per_quarter,
                    ))),
                });
            }
            MtxtOutputRecord::TimeSignature { signature, .. } => {
                let (numerator, denominator) = time_signature_to_midi(signature);

                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(delta_tick as u32),
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
                    delta: midly::num::u28::new(delta_tick as u32),
                    kind: TrackEventKind::Meta(kind),
                });
            }
            MtxtOutputRecord::Beat { .. } => {}
            MtxtOutputRecord::SysEx { data, .. } => {
                track_events.push(TrackEvent {
                    delta: midly::num::u28::new(delta_tick as u32),
                    kind: TrackEventKind::SysEx(data),
                });
            }
        }
    }

    // track_events.sort_by_key(|event| event.delta.as_int());

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

use crate::file::MtxtFile;
use crate::types::output_record::MtxtOutputRecord;
use crate::types::record::VoiceList;
use anyhow::{Result, bail};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::path::PathBuf;

use super::escape::unescape_string;
use super::instruments::INSTRUMENTS;
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

fn voice_to_program_change(voice: &VoiceList) -> u8 {
    for voice in voice.voices.iter().rev() {
        let voice_lower = voice.to_lowercase();
        if let Some(instr) = INSTRUMENTS.iter().find(|i| {
            i.mtxt_name.to_lowercase() == voice_lower || i.gm_name.to_lowercase() == voice_lower
        }) {
            return instr.gm_number;
        }

        if let Ok(num) = voice.parse::<u8>() {
            return num;
        }
    }

    0
}

fn record_to_track_event(
    record: &mut MtxtOutputRecord,
    delta_tick: u32,
) -> Result<Option<TrackEvent<'_>>> {
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

            Ok(Some(TrackEvent {
                delta: midly::num::u28::new(delta_tick),
                kind: TrackEventKind::Midi {
                    channel: midly::num::u4::new(ch),
                    message: MidiMessage::NoteOn {
                        key: midly::num::u7::new(note_num),
                        vel: midly::num::u7::new(vel),
                    },
                },
            }))
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

            Ok(Some(TrackEvent {
                delta: midly::num::u28::new(delta_tick),
                kind: TrackEventKind::Midi {
                    channel: midly::num::u4::new(ch),
                    message: MidiMessage::NoteOff {
                        key: midly::num::u7::new(note_num),
                        vel: midly::num::u7::new(vel),
                    },
                },
            }))
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

            match controller_name_to_midi(controller, *value)? {
                MidiControllerEvent::CC { number, value } => Ok(Some(TrackEvent {
                    delta: midly::num::u28::new(delta_tick),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::new(ch),
                        message: MidiMessage::Controller {
                            controller: midly::num::u7::new(number),
                            value: midly::num::u7::new(value),
                        },
                    },
                })),
                MidiControllerEvent::PitchBend { value } => Ok(Some(TrackEvent {
                    delta: midly::num::u28::new(delta_tick),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::new(ch),
                        message: MidiMessage::PitchBend {
                            bend: midly::PitchBend(midly::num::u14::new(value)),
                        },
                    },
                })),
                MidiControllerEvent::Aftertouch { value } => Ok(Some(TrackEvent {
                    delta: midly::num::u28::new(delta_tick),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::new(ch),
                        message: MidiMessage::ChannelAftertouch {
                            vel: midly::num::u7::new(value),
                        },
                    },
                })),
            }
        }
        MtxtOutputRecord::Voice {
            voices, channel, ..
        } => {
            let program = voice_to_program_change(voices);

            if program > 127 {
                bail!("Program number out of range for MIDI");
            }

            if *channel > 15 {
                bail!("Channel {} out of range for MIDI", *channel);
            }

            let ch = *channel as u8;

            Ok(Some(TrackEvent {
                delta: midly::num::u28::new(delta_tick),
                kind: TrackEventKind::Midi {
                    channel: midly::num::u4::new(ch),
                    message: MidiMessage::ProgramChange {
                        program: midly::num::u7::new(program),
                    },
                },
            }))
        }
        MtxtOutputRecord::Tempo { bpm, .. } => {
            let microseconds_per_quarter = (60_000_000.0 / *bpm) as u32;

            Ok(Some(TrackEvent {
                delta: midly::num::u28::new(delta_tick),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(midly::num::u24::new(
                    microseconds_per_quarter,
                ))),
            }))
        }
        MtxtOutputRecord::TimeSignature { signature, .. } => {
            let (numerator, denominator) = time_signature_to_midi(signature);

            Ok(Some(TrackEvent {
                delta: midly::num::u28::new(delta_tick),
                kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                    numerator,
                    denominator,
                    24, // MIDI clocks per metronome click
                    8,  // 32nd notes per quarter note
                )),
            }))
        }
        MtxtOutputRecord::Reset { .. } => {
            // Reset events don't have a direct MIDI equivalent
            // Could send All Notes Off (CC 123) or All Sound Off (CC 120)
            // For now, just skip it
            Ok(None)
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

            Ok(Some(TrackEvent {
                delta: midly::num::u28::new(delta_tick),
                kind: TrackEventKind::Meta(kind),
            }))
        }
        MtxtOutputRecord::Beat { .. } => Ok(None),
        MtxtOutputRecord::SysEx { data, .. } => Ok(Some(TrackEvent {
            delta: midly::num::u28::new(delta_tick),
            kind: TrackEventKind::SysEx(data),
        })),
    }
}

fn convert_output_records_to_midi(records: &mut [MtxtOutputRecord]) -> Result<Smf<'_>> {
    let ppqn = 480;
    let timing = Timing::Metrical(midly::num::u15::new(ppqn));

    let mut track_events = Vec::new();

    let mut current_bpm = 120.0;

    let mut last_micros = 0u64;
    let mut accumulated_delta_ticks = 0u64;

    for record in records.iter_mut() {
        let time_micros = record.time();
        let delta_micros = time_micros.saturating_sub(last_micros);
        last_micros = time_micros;

        let micros_per_beat = 60_000_000.0 / current_bpm;
        let delta_beats = delta_micros as f64 / micros_per_beat;
        let mut delta_tick = accumulated_delta_ticks + ((delta_beats * ppqn as f64).round() as u64);

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

        let track_event = record_to_track_event(record, delta_tick as u32)?;

        if let Some(event) = track_event {
            track_events.push(event);
            accumulated_delta_ticks = 0;
        } else {
            // did not manage to consume deltas -> accumulate
            accumulated_delta_ticks = delta_tick;
        }
    }

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

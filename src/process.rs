use crate::BeatTime;
use crate::MtxtRecord;
use crate::Note;
use crate::NoteTarget;
use crate::transitions::TransitionProcessor;
// use crate::transitions::expand_transitions;
use crate::types::output_record::MtxtOutputRecord;
use crate::types::pitch::PitchClass;
use crate::types::record::AliasDefinition;
use std::collections::HashMap;
use std::rc::Rc;

struct ProcessState {
    duration: BeatTime,
    channel: u16,
    velocity: f32,
    off_velocity: f32,
    transition_curve: f32,
    transition_interval: f32,
    aliases: HashMap<String, Rc<AliasDefinition>>,
    tuning: HashMap<PitchClass, f32>,
}

impl ProcessState {
    fn new() -> Self {
        Self {
            duration: BeatTime::from_parts(1, 0.0),
            channel: 0,
            velocity: 64.0,
            off_velocity: 0.0,
            transition_curve: 0.0,
            transition_interval: 0.01,
            aliases: HashMap::new(),
            tuning: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntermediateRecord {
    pub start_beat_time: BeatTime, // start_beat_time = end_beat_time - transition_time
    pub end_beat_time: BeatTime,
    pub record: MtxtOutputRecord,
    pub transition_curve: f32,
    pub transition_time: BeatTime,
    pub transition_interval: f32,
}

pub fn process_records(records: &[MtxtRecord]) -> Vec<MtxtOutputRecord> {
    let intermediate_output = create_intermediate_records(records);
    let mut transition_processor = TransitionProcessor::new(&intermediate_output);
    transition_processor.process_all()
}

fn create_intermediate_records(records: &[MtxtRecord]) -> Vec<IntermediateRecord> {
    let mut state = ProcessState::new();
    let mut intermediate_output = Vec::new();

    for record in records {
        match record {
            MtxtRecord::DurationDirective { duration } => state.duration = *duration,
            MtxtRecord::ChannelDirective { channel } => state.channel = *channel,
            MtxtRecord::VelocityDirective { velocity } => state.velocity = *velocity,
            MtxtRecord::OffVelocityDirective { off_velocity } => state.off_velocity = *off_velocity,
            MtxtRecord::TransitionCurveDirective { curve } => state.transition_curve = *curve,
            MtxtRecord::TransitionIntervalDirective { interval } => {
                state.transition_interval = *interval
            }
            MtxtRecord::AliasDef { value } => {
                state.aliases.insert(value.name.clone(), value.clone());
            }

            // Events
            MtxtRecord::Note {
                time,
                note,
                duration,
                velocity,
                off_velocity,
                channel,
            } => {
                let dur = duration.unwrap_or(state.duration);
                let vel = velocity.unwrap_or(state.velocity);
                let off_vel = off_velocity.unwrap_or(state.off_velocity);
                let ch = channel.unwrap_or(state.channel);

                let notes = resolve_note_target(note, &state.aliases);
                for mut n in notes {
                    if let Some(cents) = state.tuning.get(&n.pitch_class) {
                        n.cents += cents;
                    }
                    intermediate_output.push(IntermediateRecord {
                        start_beat_time: *time,
                        end_beat_time: *time,
                        record: MtxtOutputRecord::NoteOn {
                            time: 0,
                            note: n.clone(),
                            velocity: vel,
                            channel: ch,
                        },
                        transition_curve: 0.0,
                        transition_time: BeatTime::zero(),
                        transition_interval: 0.0,
                    });

                    intermediate_output.push(IntermediateRecord {
                        start_beat_time: *time + dur,
                        end_beat_time: *time + dur,
                        record: MtxtOutputRecord::NoteOff {
                            time: 0,
                            note: n,
                            off_velocity: off_vel,
                            channel: ch,
                        },
                        transition_curve: 0.0,
                        transition_time: BeatTime::zero(),
                        transition_interval: 0.0,
                    });
                }
            }

            MtxtRecord::NoteOn {
                time,
                note,
                velocity,
                channel,
            } => {
                let vel = velocity.unwrap_or(state.velocity);
                let ch = channel.unwrap_or(state.channel);
                let notes = resolve_note_target(note, &state.aliases);
                for mut n in notes {
                    if let Some(cents) = state.tuning.get(&n.pitch_class) {
                        n.cents += cents;
                    }
                    intermediate_output.push(IntermediateRecord {
                        start_beat_time: *time,
                        end_beat_time: *time,
                        record: MtxtOutputRecord::NoteOn {
                            time: 0,
                            note: n,
                            velocity: vel,
                            channel: ch,
                        },
                        transition_curve: 0.0,
                        transition_time: BeatTime::zero(),
                        transition_interval: 0.0,
                    });
                }
            }

            MtxtRecord::NoteOff {
                time,
                note,
                off_velocity,
                channel,
            } => {
                let off_vel = off_velocity.unwrap_or(state.off_velocity);
                let ch = channel.unwrap_or(state.channel);
                let notes = resolve_note_target(note, &state.aliases);
                for mut n in notes {
                    if let Some(cents) = state.tuning.get(&n.pitch_class) {
                        n.cents += cents;
                    }
                    intermediate_output.push(IntermediateRecord {
                        start_beat_time: *time,
                        end_beat_time: *time,
                        record: MtxtOutputRecord::NoteOff {
                            time: 0,
                            note: n,
                            off_velocity: off_vel,
                            channel: ch,
                        },
                        transition_curve: 0.0,
                        transition_time: BeatTime::zero(),
                        transition_interval: 0.0,
                    });
                }
            }

            MtxtRecord::ControlChange {
                time,
                note,
                controller,
                value,
                channel,
                transition_curve,
                transition_time,
                transition_interval,
            } => {
                let ch = channel.unwrap_or(state.channel);
                let t_curve = transition_curve.unwrap_or(state.transition_curve);
                let t_time = transition_time.unwrap_or(BeatTime::zero());
                let t_interval = transition_interval.unwrap_or(state.transition_interval);

                if let Some(target) = note {
                    let notes = resolve_note_target(target, &state.aliases);
                    for n in notes {
                        intermediate_output.push(IntermediateRecord {
                            start_beat_time: *time - t_time,
                            end_beat_time: *time,
                            record: MtxtOutputRecord::ControlChange {
                                time: 0,
                                note: Some(n),
                                controller: controller.clone(),
                                value: *value,
                                channel: ch,
                            },
                            transition_curve: t_curve,
                            transition_time: t_time,
                            transition_interval: t_interval,
                        });
                    }
                } else {
                    intermediate_output.push(IntermediateRecord {
                        start_beat_time: *time - t_time,
                        end_beat_time: *time,
                        record: MtxtOutputRecord::ControlChange {
                            time: 0,
                            note: None,
                            controller: controller.clone(),
                            value: *value,
                            channel: ch,
                        },
                        transition_curve: t_curve,
                        transition_time: t_time,
                        transition_interval: t_interval,
                    });
                }
            }

            MtxtRecord::Voice {
                time,
                voices,
                channel,
            } => {
                let ch = channel.unwrap_or(state.channel);
                intermediate_output.push(IntermediateRecord {
                    start_beat_time: *time,
                    end_beat_time: *time,
                    record: MtxtOutputRecord::Voice {
                        time: 0,
                        voices: voices.clone(),
                        channel: ch,
                    },
                    transition_curve: 0.0,
                    transition_time: BeatTime::zero(),
                    transition_interval: 0.0,
                });
            }

            MtxtRecord::Tempo {
                time,
                bpm,
                transition_curve,
                transition_time,
                transition_interval,
            } => {
                let t_curve = transition_curve.unwrap_or(state.transition_curve);
                let t_time = transition_time.unwrap_or(BeatTime::zero());
                let t_interval = transition_interval.unwrap_or(state.transition_interval);

                intermediate_output.push(IntermediateRecord {
                    start_beat_time: *time - t_time,
                    end_beat_time: *time,
                    record: MtxtOutputRecord::Tempo { time: 0, bpm: *bpm },
                    transition_curve: t_curve,
                    transition_time: t_time,
                    transition_interval: t_interval,
                });
            }

            MtxtRecord::TimeSignature { time, signature } => {
                intermediate_output.push(IntermediateRecord {
                    start_beat_time: *time,
                    end_beat_time: *time,
                    record: MtxtOutputRecord::TimeSignature {
                        time: 0,
                        signature: signature.clone(),
                    },
                    transition_curve: 0.0,
                    transition_time: BeatTime::zero(),
                    transition_interval: 0.0,
                });
            }

            MtxtRecord::Tuning {
                time: _,
                target,
                cents,
            } => {
                if let Ok(pitch_class) = target.parse::<PitchClass>() {
                    state.tuning.insert(pitch_class, *cents);
                }
            }

            MtxtRecord::Reset { time, target } => {
                intermediate_output.push(IntermediateRecord {
                    start_beat_time: *time,
                    end_beat_time: *time,
                    record: MtxtOutputRecord::Reset {
                        time: 0,
                        target: target.clone(),
                    },
                    transition_curve: 0.0,
                    transition_time: BeatTime::zero(),
                    transition_interval: 0.0,
                });
            }

            MtxtRecord::Meta {
                time,
                channel,
                meta_type,
                value,
            } => {
                let ch = channel.unwrap_or(state.channel);
                let t = time.unwrap_or(BeatTime::zero());
                intermediate_output.push(IntermediateRecord {
                    start_beat_time: t,
                    end_beat_time: t,
                    record: MtxtOutputRecord::ChannelMeta {
                        time: 0,
                        channel: ch,
                        meta_type: meta_type.clone(),
                        value: value.clone(),
                    },
                    transition_curve: 0.0,
                    transition_time: BeatTime::zero(),
                    transition_interval: 0.0,
                });
            }

            MtxtRecord::GlobalMeta { meta_type, value } => {
                intermediate_output.push(IntermediateRecord {
                    start_beat_time: BeatTime::zero(),
                    end_beat_time: BeatTime::zero(),
                    record: MtxtOutputRecord::GlobalMeta {
                        time: 0,
                        meta_type: meta_type.clone(),
                        value: value.clone(),
                    },
                    transition_curve: 0.0,
                    transition_time: BeatTime::zero(),
                    transition_interval: 0.0,
                });
            }

            MtxtRecord::SysEx { time, data } => {
                intermediate_output.push(IntermediateRecord {
                    start_beat_time: *time,
                    end_beat_time: *time,
                    record: MtxtOutputRecord::SysEx {
                        time: 0,
                        data: data.clone(),
                    },
                    transition_curve: 0.0,
                    transition_time: BeatTime::zero(),
                    transition_interval: 0.0,
                });
            }

            MtxtRecord::Header { version: _ } => {}
            MtxtRecord::EmptyLine | MtxtRecord::Comment { .. } => {}
        }
    }

    intermediate_output.sort_by(|a, b| a.end_beat_time.cmp(&b.end_beat_time));
    intermediate_output
}

fn resolve_note_target(
    target: &NoteTarget,
    aliases: &HashMap<String, Rc<AliasDefinition>>,
) -> Vec<Note> {
    match target {
        NoteTarget::Note(note) => vec![note.clone()],
        NoteTarget::AliasKey(name) => {
            if let Some(def) = aliases.get(name) {
                def.notes.clone()
            } else {
                vec![]
            }
        }
        NoteTarget::Alias(def) => def.notes.clone(),
    }
}

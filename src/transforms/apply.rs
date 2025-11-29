use crate::BeatTime;
use crate::types::record::MtxtRecord;

struct State {
    channel: Option<u16>,
    velocity: Option<f32>,
    off_velocity: Option<f32>,
    duration: Option<BeatTime>,
    transition_curve: Option<f32>,
    transition_interval: Option<f32>,
}

impl State {
    fn new() -> Self {
        Self {
            channel: None,
            velocity: None,
            off_velocity: None,
            duration: None,
            transition_curve: None,
            transition_interval: None,
        }
    }
}

pub fn transform(records: &[MtxtRecord]) -> Vec<MtxtRecord> {
    let mut state = State::new();
    let mut new_records = Vec::with_capacity(records.len());

    for record in records {
        match record {
            MtxtRecord::ChannelDirective { channel } => {
                state.channel = Some(*channel);
            }
            MtxtRecord::VelocityDirective { velocity } => {
                state.velocity = Some(*velocity);
            }
            MtxtRecord::OffVelocityDirective { off_velocity } => {
                state.off_velocity = Some(*off_velocity);
            }
            MtxtRecord::DurationDirective { duration } => {
                state.duration = Some(*duration);
            }
            MtxtRecord::TransitionCurveDirective { curve } => {
                state.transition_curve = Some(*curve);
            }
            MtxtRecord::TransitionIntervalDirective { interval } => {
                state.transition_interval = Some(*interval);
            }

            MtxtRecord::Note {
                time,
                note,
                duration,
                velocity,
                off_velocity,
                channel,
            } => {
                new_records.push(MtxtRecord::Note {
                    time: *time,
                    note: note.clone(),
                    duration: duration.or(state.duration),
                    velocity: velocity.or(state.velocity),
                    off_velocity: off_velocity.or(state.off_velocity),
                    channel: channel.or(state.channel),
                });
            }
            MtxtRecord::NoteOn {
                time,
                note,
                velocity,
                channel,
            } => {
                new_records.push(MtxtRecord::NoteOn {
                    time: *time,
                    note: note.clone(),
                    velocity: velocity.or(state.velocity),
                    channel: channel.or(state.channel),
                });
            }
            MtxtRecord::NoteOff {
                time,
                note,
                off_velocity,
                channel,
            } => {
                new_records.push(MtxtRecord::NoteOff {
                    time: *time,
                    note: note.clone(),
                    off_velocity: off_velocity.or(state.off_velocity),
                    channel: channel.or(state.channel),
                });
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
                new_records.push(MtxtRecord::ControlChange {
                    time: *time,
                    note: note.clone(),
                    controller: controller.clone(),
                    value: *value,
                    channel: *channel,
                    transition_curve: transition_curve.or(state.transition_curve),
                    transition_time: *transition_time,
                    transition_interval: transition_interval.or(state.transition_interval),
                });
            }
            MtxtRecord::Voice {
                time,
                voices,
                channel,
            } => {
                new_records.push(MtxtRecord::Voice {
                    time: *time,
                    voices: voices.clone(),
                    channel: channel.or(state.channel),
                });
            }
            MtxtRecord::Tempo {
                time,
                bpm,
                transition_curve,
                transition_time,
                transition_interval,
            } => {
                new_records.push(MtxtRecord::Tempo {
                    time: *time,
                    bpm: *bpm,
                    transition_curve: transition_curve.or(state.transition_curve),
                    transition_time: *transition_time,
                    transition_interval: transition_interval.or(state.transition_interval),
                });
            }
            _ => {
                new_records.push(record.clone());
            }
        }
    }

    new_records
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::assert_eq_records;

    #[test]
    fn test_apply_directives() {
        let input = r#"
mtxt 1.0
ch=1
vel=0.8
dur=1
1.0 note C4
2.0 note E4 dur=2
3.0 note G4 vel=0.5
ch=2
4.0 note C5
transition_curve=0.5
5.0 cc volume 1.0
"#;
        let expected = r#"
mtxt 1.0
1.0 note C4 dur=1 vel=0.8 ch=1
2.0 note E4 dur=2 vel=0.8 ch=1
3.0 note G4 dur=1 vel=0.5 ch=1
4.0 note C5 dur=1 vel=0.8 ch=2
5.0 cc volume 1 transition_curve=0.5
"#;

        assert_eq_records(input, transform, expected);
    }
}

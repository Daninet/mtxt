use crate::transforms::apply;
use crate::types::record::MtxtRecord;

/// Extracts common inline parameters into global directives.
fn extract_property<T: PartialEq + Clone + Copy + std::fmt::Debug>(
    records: Vec<MtxtRecord>,
    get_fn: impl Fn(&MtxtRecord) -> Option<T>,
    create_directive_fn: impl Fn(T) -> MtxtRecord,
    remove_fn: impl Fn(&mut MtxtRecord),
) -> Vec<MtxtRecord> {
    let mut result = Vec::new();
    let mut current_global_value: Option<T> = None;
    let mut i = 0;
    while i < records.len() {
        let rec = &records[i];

        // Check if current record has the property explicitly set
        if let Some(val) = get_fn(rec) {
            // Check if it matches the current global value
            if let Some(global) = current_global_value {
                if val == global {
                    // Matches global, just remove inline property
                    let mut new_rec = rec.clone();
                    remove_fn(&mut new_rec);
                    result.push(new_rec);
                    i += 1;
                    continue;
                }
            }

            // Start looking ahead for a run
            let mut run_indices = vec![i];
            let mut j = i + 1;

            while j < records.len() {
                let next_rec = &records[j];

                if let Some(next_val) = get_fn(next_rec) {
                    if next_val == val {
                        run_indices.push(j);
                    } else {
                        // Different explicit value -> break run
                        break;
                    }
                } else {
                    // No value (None) -> skip (transparent)
                }
                j += 1;
            }

            if run_indices.len() >= 3 {
                // Found a run of at least 3
                result.push(create_directive_fn(val));
                current_global_value = Some(val);

                // Process the block from i to j
                for k in i..j {
                    let mut r = records[k].clone();
                    if run_indices.contains(&k) {
                        remove_fn(&mut r);
                    }
                    result.push(r);
                }
                i = j;
            } else {
                // Not enough for a run, just push the current record
                result.push(rec.clone());
                i += 1;
            }
        } else {
            // No explicit property, just push
            result.push(rec.clone());
            i += 1;
        }
    }
    result
}

pub fn transform(records: &[MtxtRecord]) -> Vec<MtxtRecord> {
    // Step 1: Apply all directives to make everything inline
    // This removes all existing directives and propagates their values to the events
    let mut current = apply::transform(records);

    // Step 2: Extract properties one by one
    current = extract_property(
        current,
        |r| match r {
            MtxtRecord::Note { channel, .. }
            | MtxtRecord::NoteOn { channel, .. }
            | MtxtRecord::NoteOff { channel, .. }
            | MtxtRecord::Voice { channel, .. } => *channel,
            _ => None,
        },
        |v| MtxtRecord::ChannelDirective { channel: v },
        |r| match r {
            MtxtRecord::Note { channel, .. }
            | MtxtRecord::NoteOn { channel, .. }
            | MtxtRecord::NoteOff { channel, .. }
            | MtxtRecord::Voice { channel, .. } => *channel = None,
            _ => {}
        },
    );

    current = extract_property(
        current,
        |r| match r {
            MtxtRecord::Note { velocity, .. } | MtxtRecord::NoteOn { velocity, .. } => *velocity,
            _ => None,
        },
        |v| MtxtRecord::VelocityDirective { velocity: v },
        |r| match r {
            MtxtRecord::Note { velocity, .. } | MtxtRecord::NoteOn { velocity, .. } => {
                *velocity = None
            }
            _ => {}
        },
    );

    current =
        extract_property(
            current,
            |r| match r {
                MtxtRecord::Note { off_velocity, .. }
                | MtxtRecord::NoteOff { off_velocity, .. } => *off_velocity,
                _ => None,
            },
            |v| MtxtRecord::OffVelocityDirective { off_velocity: v },
            |r| match r {
                MtxtRecord::Note { off_velocity, .. }
                | MtxtRecord::NoteOff { off_velocity, .. } => *off_velocity = None,
                _ => {}
            },
        );

    current = extract_property(
        current,
        |r| match r {
            MtxtRecord::Note { duration, .. } => *duration,
            _ => None,
        },
        |v| MtxtRecord::DurationDirective { duration: v },
        |r| match r {
            MtxtRecord::Note { duration, .. } => *duration = None,
            _ => {}
        },
    );

    current = extract_property(
        current,
        |r| match r {
            MtxtRecord::ControlChange {
                transition_curve, ..
            }
            | MtxtRecord::Tempo {
                transition_curve, ..
            } => *transition_curve,
            _ => None,
        },
        |v| MtxtRecord::TransitionCurveDirective { curve: v },
        |r| match r {
            MtxtRecord::ControlChange {
                transition_curve, ..
            }
            | MtxtRecord::Tempo {
                transition_curve, ..
            } => *transition_curve = None,
            _ => {}
        },
    );

    current = extract_property(
        current,
        |r| match r {
            MtxtRecord::ControlChange {
                transition_interval,
                ..
            }
            | MtxtRecord::Tempo {
                transition_interval,
                ..
            } => *transition_interval,
            _ => None,
        },
        |v| MtxtRecord::TransitionIntervalDirective { interval: v },
        |r| match r {
            MtxtRecord::ControlChange {
                transition_interval,
                ..
            }
            | MtxtRecord::Tempo {
                transition_interval,
                ..
            } => *transition_interval = None,
            _ => {}
        },
    );

    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::assert_eq_records;

    #[test]
    fn test_extract_directives() {
        let input = r#"
mtxt 1.0
1.0 note C4 ch=1
2.0 note E4 ch=1
3.0 note G4 ch=1

4.0 note C5 ch=2
5.0 note E5 ch=2
6.0 note G5 ch=2
7.0 note C6 ch=3
8.0 note G5 ch=1
9.0 note G5 ch=2
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note C4
2.0 note E4
3.0 note G4

ch=2
4.0 note C5
5.0 note E5
6.0 note G5
7.0 note C6 ch=3
8.0 note G5 ch=1
9.0 note G5
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_extract_directives_mixed() {
        let input = r#"
mtxt 1.0
1.0 note C4 ch=1 vel=0.5
2.0 note E4 ch=1 vel=0.5
2.5 tempo 120
// comment
3.0 note G4 ch=1 vel=0.5
"#;
        let expected = r#"
mtxt 1.0
ch=1
vel=0.5
1.0 note C4
2.0 note E4
2.5 tempo 120
// comment
3.0 note G4
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_extract_directives_interrupted() {
        let input = r#"
mtxt 1.0
ch=1
1.0 note C4 ch=1
2.0 note E4
3.0 note G4 ch=1
4.0 note C5 ch=2
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note C4
2.0 note E4
3.0 note G4
4.0 note C5 ch=2
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_extract_directives_ignore_cc_for_channel() {
        let input = r#"
mtxt 1.0
1.0 cc ch=1 volume 1
1.0 note C4 ch=1
2.0 cc ch=2 volume 0.9
3.0 note E4 ch=1
3.5 cc volume 0.8
4.0 note G4 ch=1
"#;
        let expected = r#"
mtxt 1.0
1.0 cc ch=1 volume 1
ch=1
1.0 note C4
2.0 cc ch=2 volume 0.9
3.0 note E4
3.5 cc volume 0.8
4.0 note G4
"#;
        assert_eq_records(input, transform, expected);
    }
}

use crate::types::note::{Note, NoteTarget};
use crate::types::record::{AliasDefinition, MtxtRecord};
use std::collections::HashMap;
use std::rc::Rc;

fn transpose_target(
    target: &NoteTarget,
    amount: i32,
    map: &HashMap<usize, Rc<AliasDefinition>>,
) -> NoteTarget {
    match target {
        NoteTarget::Note(n) => NoteTarget::Note(n.transpose(amount)),
        NoteTarget::AliasKey(k) => NoteTarget::AliasKey(k.clone()),
        NoteTarget::Alias(rc) => {
            let ptr = Rc::as_ptr(rc) as usize;
            if let Some(new_rc) = map.get(&ptr) {
                NoteTarget::Alias(new_rc.clone())
            } else {
                // If not found, it means the alias def was not in the file or not yet seen.
                // We return the original.
                NoteTarget::Alias(rc.clone())
            }
        }
    }
}

pub fn transform(records: &[MtxtRecord], amount: i32) -> Vec<MtxtRecord> {
    if amount == 0 {
        return records.to_vec();
    }

    let mut new_records = Vec::with_capacity(records.len());
    let mut alias_map: HashMap<usize, Rc<AliasDefinition>> = HashMap::new();

    for record in records {
        let new_record = match record {
            MtxtRecord::AliasDef { value } => {
                let new_notes: Vec<Note> =
                    value.notes.iter().map(|n| n.transpose(amount)).collect();
                let new_def = Rc::new(AliasDefinition {
                    name: value.name.clone(),
                    notes: new_notes,
                });
                alias_map.insert(Rc::as_ptr(value) as usize, new_def.clone());
                MtxtRecord::AliasDef { value: new_def }
            }
            MtxtRecord::Note {
                time,
                note,
                duration,
                velocity,
                off_velocity,
                channel,
            } => MtxtRecord::Note {
                time: *time,
                note: transpose_target(note, amount, &alias_map),
                duration: *duration,
                velocity: *velocity,
                off_velocity: *off_velocity,
                channel: *channel,
            },
            MtxtRecord::NoteOn {
                time,
                note,
                velocity,
                channel,
            } => MtxtRecord::NoteOn {
                time: *time,
                note: transpose_target(note, amount, &alias_map),
                velocity: *velocity,
                channel: *channel,
            },
            MtxtRecord::NoteOff {
                time,
                note,
                off_velocity,
                channel,
            } => MtxtRecord::NoteOff {
                time: *time,
                note: transpose_target(note, amount, &alias_map),
                off_velocity: *off_velocity,
                channel: *channel,
            },
            MtxtRecord::ControlChange {
                time,
                note,
                controller,
                value,
                channel,
                transition_curve,
                transition_time,
                transition_interval,
            } => MtxtRecord::ControlChange {
                time: *time,
                note: note
                    .as_ref()
                    .map(|n| transpose_target(n, amount, &alias_map)),
                controller: controller.clone(),
                value: *value,
                channel: *channel,
                transition_curve: *transition_curve,
                transition_time: *transition_time,
                transition_interval: *transition_interval,
            },
            _ => record.clone(),
        };
        new_records.push(new_record);
    }
    new_records
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::assert_eq_records;

    #[test]
    fn test_transpose() {
        let input = r#"
mtxt 1.0
alias Cmaj C4,E4,G4
1.0 note C4+2 dur=1
2.0 note Cmaj dur=2
3.0 cc C2 volume 0.5
"#;
        let expected = r#"
mtxt 1.0
alias Cmaj B2,Eb3,F#3
1.0 note B2+2 dur=1
2.0 note Cmaj dur=2
3.0 cc B0 volume 0.5
"#;

        assert_eq_records(input, |records| transform(records, -13), expected);
    }
}

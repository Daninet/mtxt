use crate::types::note::NoteTarget;
use crate::types::record::MtxtRecord;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum NoteKey {
    Note(i32, u32), // total semitone, cents as u32 bits
    Alias(String),
}

fn get_key(target: &NoteTarget) -> NoteKey {
    match target {
        NoteTarget::Note(n) => {
            let semitone = (n.octave as i32 + 1) * 12 + n.pitch_class.to_semitone() as i32;
            NoteKey::Note(semitone, n.cents.to_bits())
        }
        NoteTarget::AliasKey(s) => NoteKey::Alias(s.clone()),
        NoteTarget::Alias(def) => NoteKey::Alias(def.name.clone()),
    }
}

pub fn transform(records: &[MtxtRecord]) -> Vec<MtxtRecord> {
    let mut new_records = Vec::new();
    // Key: (effective_channel, note_key) -> index in new_records
    let mut pending: HashMap<(u16, NoteKey), usize> = HashMap::new();
    let mut current_channel: u16 = 0;

    for record in records {
        // Track channel directives
        if let MtxtRecord::ChannelDirective { channel } = record {
            current_channel = *channel;
        }

        match record {
            MtxtRecord::NoteOn {
                time: _,
                note,
                velocity: _,
                channel,
            } => {
                let eff_ch = channel.unwrap_or(current_channel);
                let key = get_key(note);

                // If we already have a pending for this key, we leave it as NoteOn
                // and start a new one. This handles polyphony/retrigger if allowed,
                // or just error recovery.
                // Better strategy: overwrite pending with new index.
                let idx = new_records.len();
                pending.insert((eff_ch, key), idx);
                new_records.push(record.clone());
            }
            MtxtRecord::NoteOff {
                time: off_time,
                note,
                off_velocity,
                channel,
            } => {
                let eff_ch = channel.unwrap_or(current_channel);
                let key = get_key(note);

                if let Some(idx) = pending.remove(&(eff_ch, key)) {
                    if let Some(MtxtRecord::NoteOn {
                        time: on_time,
                        note: _,
                        velocity,
                        channel: on_channel,
                    }) = new_records.get(idx)
                    {
                        let duration = *off_time - *on_time;
                        // Create merged Note
                        let new_note = MtxtRecord::Note {
                            time: *on_time,
                            note: note.clone(),
                            duration: Some(duration),
                            velocity: *velocity,
                            off_velocity: *off_velocity,
                            channel: *on_channel,
                        };
                        new_records[idx] = new_note;
                    }
                } else {
                    // Unmatched NoteOff
                    new_records.push(record.clone());
                }
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
    fn test_merge_notes() {
        let input = r#"
mtxt 1.0
ch=1
1.0 on C4 vel=0.5
2.0 off C4 offvel=0.8
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note C4 dur=1.0 vel=0.5 offvel=0.8
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_merge_notes_interleaved() {
        let input = r#"
mtxt 1.0
ch=1
1.0 on C4
1.5 on E4
2.0 off C4
3.5 off E4
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note C4 dur=1.0
1.5 note E4 dur=2.0
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_merge_channel_handling() {
        let input = r#"
mtxt 1.0
ch=1
1.0 on C4
ch=2
1.0 on C4
2.0 off C4
ch=1
3.0 off C4
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note C4 dur=2.0
ch=2
1.0 note C4 dur=1.0
ch=1
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_unmatched_note_on() {
        let input = r#"
mtxt 1.0
1.0 on C4
"#;
        let expected = r#"
mtxt 1.0
1.0 on C4
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_unmatched_note_off() {
        let input = r#"
mtxt 1.0
1.0 off C4
"#;
        let expected = r#"
mtxt 1.0
1.0 off C4
"#;
        assert_eq_records(input, transform, expected);
    }
}

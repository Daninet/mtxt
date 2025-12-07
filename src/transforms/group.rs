use crate::transforms::{apply, extract};
use crate::types::record::MtxtRecord;
use std::cmp::Ordering;

fn get_channel(record: &MtxtRecord) -> Option<u16> {
    match record {
        MtxtRecord::Note { channel, .. }
        | MtxtRecord::NoteOn { channel, .. }
        | MtxtRecord::NoteOff { channel, .. }
        | MtxtRecord::Voice { channel, .. } => *channel,
        MtxtRecord::ControlChange { channel, .. } => *channel,
        _ => None,
    }
}

pub fn transform(records: &[MtxtRecord]) -> Vec<MtxtRecord> {
    // 1. Apply directives to flatten state
    let mut current_records = apply::transform(records);

    // 2. Sort by Channel then Time
    current_records.sort_by(|a, b| {
        let ch_a = get_channel(a);
        let ch_b = get_channel(b);

        match ch_a.cmp(&ch_b) {
            Ordering::Equal => {
                let time_a = a.time();
                let time_b = b.time();
                match (time_a, time_b) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some(ta), Some(tb)) => ta.cmp(&tb),
                }
            }
            ord => ord,
        }
    });

    // 3. Extract directives to re-group
    extract::transform(&current_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::assert_eq_records;

    #[test]
    fn test_group_channels() {
        let input = r#"
mtxt 1.0
1.0 note C4 ch=1
1.5 note C4 ch=2
3.0 note G4 ch=1
2.5 note E4 ch=2
3.5 note G4 ch=2
2.0 note E4 ch=1
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note C4
2.0 note E4
3.0 note G4
ch=2
1.5 note C4
2.5 note E4
3.5 note G4
"#;
        assert_eq_records(input, transform, expected);
    }

    #[test]
    fn test_group_channels_with_globals() {
        let input = r#"
mtxt 1.0
0.5 tempo 120
ch=1
1.0 on C4
ch=2
1.5 note D4
1.5 note H4
3.0 note F4 ch=1
ch=1
2.0 note E4
1.0 note G4 ch=3
"#;
        let expected = r#"
mtxt 1.0
0.5 tempo 120
ch=1
1.0 on C4
2.0 note E4
3.0 note F4
1.5 note D4 ch=2
1.5 note H4 ch=2
1.0 note G4 ch=3
"#;
        assert_eq_records(input, transform, expected);
    }
}

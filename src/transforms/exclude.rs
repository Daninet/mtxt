use crate::types::record::MtxtRecord;
use std::collections::HashSet;

pub fn transform(records: &[MtxtRecord], channels: &HashSet<u16>) -> Vec<MtxtRecord> {
    if channels.is_empty() {
        return records.to_vec();
    }

    let mut current_channel: Option<u16> = None;

    records
        .iter()
        .filter(|record| match record {
            MtxtRecord::Note { channel, .. }
            | MtxtRecord::NoteOn { channel, .. }
            | MtxtRecord::NoteOff { channel, .. }
            | MtxtRecord::Voice { channel, .. } => {
                if let Some(channel) = channel {
                    !channels.contains(channel)
                } else if let Some(curr) = current_channel {
                    !channels.contains(&curr)
                } else {
                    false
                }
            }
            MtxtRecord::ControlChange { channel, .. } => {
                // if channel is None, affects all channels
                channel.is_none_or(|ch| !channels.contains(&ch))
            }
            MtxtRecord::ChannelDirective { channel } => {
                current_channel = Some(*channel);
                !channels.contains(channel)
            }
            _ => true,
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::assert_eq_records;

    fn exclude_channels_1_2(records: &[MtxtRecord]) -> Vec<MtxtRecord> {
        transform(records, &HashSet::from([1, 2]))
    }

    #[test]
    fn test_exclude_channels() {
        let input = r#"
mtxt 1.0
ch=1
0.0 voice piano
0.0 voice ch=3 trombone
1.0 note C4 dur=1 ch=1
2.0 note E4 dur=1 ch=2
3.0 note G4 dur=1
4.0 note F5 dur=1 ch=3
4.0 cc volume 1
ch=5
5.0 note A5 dur=1
5.0 cc C4 volume 0.5 ch=1
6.0 cc E4 volume 0.5 ch=2
7.0 cc G4 volume 0.5
"#;
        let expected = r#"
mtxt 1.0
0.0 voice ch=3 trombone
4.0 note F5 dur=1 ch=3
4.0 cc volume 1
ch=5
5.0 note A5 dur=1
7.0 cc G4 volume 0.5
"#;

        assert_eq_records(input, exclude_channels_1_2, expected);
    }
}

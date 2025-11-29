use crate::types::record::MtxtRecord;
use std::cmp::Ordering;

pub fn transform(records: &[MtxtRecord]) -> Vec<MtxtRecord> {
    let mut new_records = Vec::with_capacity(records.len());
    let mut buffer: Vec<MtxtRecord> = Vec::new();

    for record in records {
        if record.time().is_some() {
            buffer.push(record.clone());
        } else {
            // Barrier encountered: sort and flush buffer
            if !buffer.is_empty() {
                buffer.sort_by(|a, b| {
                    let ta = a.time().unwrap();
                    let tb = b.time().unwrap();
                    ta.partial_cmp(&tb).unwrap_or(Ordering::Equal)
                });
                new_records.append(&mut buffer);
            }
            // Push the barrier record
            new_records.push(record.clone());
        }
    }

    // Flush remaining buffer
    if !buffer.is_empty() {
        buffer.sort_by(|a, b| {
            let ta = a.time().unwrap();
            let tb = b.time().unwrap();
            ta.partial_cmp(&tb).unwrap_or(Ordering::Equal)
        });
        new_records.append(&mut buffer);
    }

    new_records
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::assert_eq_records;

    #[test]
    fn test_sort_by_time() {
        let input = r#"
mtxt 1.0
ch=1
2.0 note C4
1.0 note E4
3.0 note G4
ch=2
5.0 note C5
4.0 note E5
// comment
7.0 note G5
6.0 note C6
"#;
        let expected = r#"
mtxt 1.0
ch=1
1.0 note E4
2.0 note C4
3.0 note G4
ch=2
4.0 note E5
5.0 note C5
// comment
6.0 note C6
7.0 note G5
"#;

        assert_eq_records(input, transform, expected);
    }
}

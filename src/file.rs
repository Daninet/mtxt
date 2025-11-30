use crate::process::process_records;
use crate::types::beat_time::BeatTime;
use crate::types::output_record::MtxtOutputRecord;
use crate::types::record::MtxtRecord;
use crate::types::version::Version;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MtxtFile {
    pub records: Vec<MtxtRecord>,
}

impl Default for MtxtFile {
    fn default() -> Self {
        Self::new()
    }
}

impl MtxtFile {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn from_records(records: Vec<MtxtRecord>) -> Self {
        Self { records }
    }

    /// Get all records
    pub fn get_records(&self) -> &[MtxtRecord] {
        &self.records
    }

    /// Get the version from the records
    pub fn get_version(&self) -> Option<&Version> {
        self.records.iter().find_map(|record| match record {
            MtxtRecord::Header { version } => Some(version),
            _ => None,
        })
    }

    pub fn get_global_meta(&self) -> Vec<(&str, &str)> {
        self.records
            .iter()
            .filter_map(|record| match record {
                MtxtRecord::GlobalMeta { meta_type, value } => {
                    Some((meta_type.as_str(), value.as_str()))
                }
                _ => None,
            })
            .collect()
    }

    /// Get a specific global meta value
    pub fn get_global_meta_value(&self, meta_type: &str) -> Option<&str> {
        self.records.iter().find_map(|record| match record {
            MtxtRecord::GlobalMeta {
                meta_type: mt,
                value,
            } if mt == meta_type => Some(value.as_str()),
            _ => None,
        })
    }

    pub fn duration(&self) -> Option<BeatTime> {
        fn record_time(record: &MtxtRecord) -> Option<BeatTime> {
            record.time()
        }

        self.records
            .iter()
            .fold(None, |max, rec| match (max, record_time(rec)) {
                (Some(m), Some(t)) if t <= m => Some(m),
                (Some(_), None) => max,
                (_, Some(t)) => Some(t),
                (None, None) => None,
            })
    }

    pub fn add_global_meta(&mut self, meta_type: String, value: String) {
        self.records
            .push(MtxtRecord::GlobalMeta { meta_type, value });
    }

    pub fn get_output_records(&self) -> Vec<MtxtOutputRecord> {
        process_records(&self.records)
    }
}

impl fmt::Display for MtxtFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0usize;
        while i < self.records.len() {
            let record = &self.records[i];
            match record {
                // File-level records don't have timestamps
                MtxtRecord::Header { .. } | MtxtRecord::GlobalMeta { .. } => {
                    writeln!(f, "{}", record)?;
                }
                // Formatting-only records
                MtxtRecord::EmptyLine => {
                    writeln!(f)?;
                }
                MtxtRecord::Comment { text } => {
                    writeln!(f, "// {}", text)?;
                }
                // Timed or directive records: print with timestamp
                _ => {
                    let time = record.time();

                    let with_time = match time {
                        Some(time) => format!("{} {}", time, record),
                        None => format!("{}", record),
                    };

                    writeln!(f, "{}", with_time)?;
                }
            }
            i += 1;
        }
        Ok(())
    }
}

use crate::process::process_records;
use crate::types::beat_time::BeatTime;
use crate::types::output_record::MtxtOutputRecord;
use crate::types::record::{MtxtRecord, MtxtRecordLine};
use crate::types::version::Version;
use std::fmt;

pub struct MtxtFileFormatter<'a> {
    file: &'a MtxtFile,
    timestamp_width: Option<usize>,
}

impl<'a> fmt::Display for MtxtFileFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.file.records {
            let record = &line.record;
            match record {
                // File-level records don't have timestamps
                MtxtRecord::Header { .. } | MtxtRecord::GlobalMeta { .. } => {
                    write!(f, "{}", record)?;
                }
                // Formatting-only records
                MtxtRecord::EmptyLine => {
                    if let Some(comment) = &line.comment {
                        write!(f, "// {}", comment)?;
                    }
                }
                // Timed or directive records: print with timestamp
                _ => {
                    match record.time() {
                        Some(time) => {
                            if let Some(width) = self.timestamp_width {
                                write!(f, "{:<width$} {}", time, record, width = width)?;
                            } else {
                                write!(f, "{} {}", time, record)?;
                            }
                        }
                        None => {
                            write!(f, "{}", record)?;
                        }
                    };
                }
            }

            if record != &MtxtRecord::EmptyLine {
                if let Some(comment) = &line.comment {
                    write!(f, " // {}", comment)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MtxtFile {
    pub records: Vec<MtxtRecordLine>,
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

    pub fn from_records(records: Vec<MtxtRecordLine>) -> Self {
        Self { records }
    }

    /// Get all records (underlying MtxtRecord values)
    pub fn get_records(&self) -> Vec<&MtxtRecord> {
        self.records.iter().map(|line| &line.record).collect()
    }

    /// Get the version from the records
    pub fn get_version(&self) -> Option<&Version> {
        self.records.iter().find_map(|line| match &line.record {
            MtxtRecord::Header { version } => Some(version),
            _ => None,
        })
    }

    pub fn get_global_meta(&self) -> Vec<(&str, &str)> {
        self.records
            .iter()
            .filter_map(|line| match &line.record {
                MtxtRecord::GlobalMeta { meta_type, value } => {
                    Some((meta_type.as_str(), value.as_str()))
                }
                _ => None,
            })
            .collect()
    }

    /// Get a specific global meta value
    pub fn get_global_meta_value(&self, meta_type: &str) -> Option<&str> {
        self.records.iter().find_map(|line| match &line.record {
            MtxtRecord::GlobalMeta {
                meta_type: mt,
                value,
            } if mt == meta_type => Some(value.as_str()),
            _ => None,
        })
    }

    pub fn duration(&self) -> Option<BeatTime> {
        self.records
            .iter()
            .fold(None, |max, line| match (max, line.record.time()) {
                (Some(m), Some(t)) if t <= m => Some(m),
                (Some(_), None) => max,
                (_, Some(t)) => Some(t),
                (None, None) => None,
            })
    }

    pub fn add_global_meta(&mut self, meta_type: String, value: String) {
        self.records
            .push(MtxtRecordLine::new(MtxtRecord::GlobalMeta {
                meta_type,
                value,
            }));
    }

    pub fn calculate_auto_timestamp_width(&self) -> usize {
        let max_time = self.duration().unwrap_or(BeatTime::zero());
        let digits = max_time.whole_beats().to_string().len();
        // digits + 1 (dot) + 5 (fractional digits)
        digits + 1 + 5
    }

    pub fn get_output_records(&self) -> Vec<MtxtOutputRecord> {
        let records: Vec<MtxtRecord> = self
            .records
            .iter()
            .map(|line| line.record.clone())
            .collect();
        process_records(&records)
    }

    pub fn display_with_formatting<'a>(
        &'a self,
        timestamp_width: Option<usize>,
    ) -> MtxtFileFormatter<'a> {
        MtxtFileFormatter {
            file: self,
            timestamp_width,
        }
    }
}

impl fmt::Display for MtxtFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_with_formatting(None))
    }
}

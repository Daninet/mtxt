use crate::BeatTime;
use crate::process::IntermediateRecord;
use crate::types::output_record::MtxtOutputRecord;
use std::cmp::Ordering;
use std::collections::HashMap;

/// - curve > 0: ease-in (starts slow, ends fast)
/// - curve < 0: ease-out (starts fast, ends slow)
/// - curve = 0: linear interpolation
fn apply_transition_curve(v0: f32, v1: f32, pos: f32, curve: f32) -> f32 {
    v0 + (v1 - v0)
        * (pos + curve.max(0.0) * (pos.powi(4) - pos)
            - (-curve).max(0.0) * ((1.0 - (1.0 - pos).powi(4)) - pos))
}

#[derive(Clone)]
struct ActiveTransition {
    start_value: f32,
    end_value: f32,
    next_pos: f32,
    next_micros: u64,
    original_record: IntermediateRecord,
}

pub struct TransitionProcessor {
    active_transitions: Vec<ActiveTransition>,
    records: Vec<IntermediateRecord>,
    next_record_idx: usize,
    current_micros: u64,
    current_beat_time: BeatTime,
    current_bpm: f32,
    last_values: HashMap<String, f32>,
    next_beat_to_emit: BeatTime,
}

impl TransitionProcessor {
    pub fn new(records: &[IntermediateRecord]) -> Self {
        let mut sorted_records = records.to_vec();
        sorted_records.sort_by(|a, b| match a.start_beat_time.cmp(&b.start_beat_time) {
            Ordering::Equal => a.transition_time.cmp(&b.transition_time),
            other => other,
        });

        Self {
            active_transitions: Vec::new(),
            records: sorted_records,
            next_record_idx: 0,

            current_micros: 0,
            current_bpm: 120.0,
            current_beat_time: BeatTime::zero(),
            last_values: HashMap::new(),
            next_beat_to_emit: BeatTime::zero(),
        }
    }

    fn consume_record(&mut self, next_record_micros: u64) -> Option<MtxtOutputRecord> {
        let record = self.records.get(self.next_record_idx).unwrap().clone();

        self.active_transitions
            .retain(|t| !t.original_record.record.is_same_parameter(&record.record));

        if record.start_beat_time != record.end_beat_time {
            let key = record.record.get_param_key().unwrap();

            let start_value = *self
                .last_values
                .get(&key)
                .unwrap_or_else(|| panic!("Error getting key {}", key));

            let next_pos = (record.transition_interval * 1000.0)
                / (record.transition_time.as_micros(self.current_bpm as f64) as f32);

            let next_micros =
                next_record_micros + (record.transition_interval * 1000.0).round() as u64;

            let transition = ActiveTransition {
                start_value,
                end_value: record.record.get_parameter_value().unwrap(),
                next_pos,
                next_micros,
                original_record: record.clone(),
            };
            self.active_transitions.push(transition);
            self.current_micros = next_record_micros;

            self.current_beat_time = record.start_beat_time;
            self.next_record_idx += 1;

            return self.process();
        }

        self.current_micros = next_record_micros;

        self.current_beat_time = record.start_beat_time;
        self.next_record_idx += 1;

        let mut res = record.record.clone();
        res.set_time(self.current_micros);
        Some(res)
    }

    fn consume_transition(&mut self, transition_idx: usize) -> Option<MtxtOutputRecord> {
        let transition = &mut self.active_transitions[transition_idx];
        let mut res = transition.original_record.record.clone();
        let diff_micros = transition.next_micros - self.current_micros;
        self.current_micros = transition.next_micros;
        self.current_beat_time =
            self.current_beat_time + BeatTime::from_micros(diff_micros, self.current_bpm as f64);

        res.set_time(self.current_micros);

        let new_value = apply_transition_curve(
            transition.start_value,
            transition.end_value,
            transition.next_pos.min(1.0),
            transition.original_record.transition_curve,
        );

        res.set_parameter_value(new_value);

        if transition.next_pos >= 1.0 {
            self.active_transitions.remove(transition_idx);
        } else {
            let bpm = if matches!(
                transition.original_record.record,
                MtxtOutputRecord::Tempo { .. }
            ) {
                new_value
            } else {
                self.current_bpm
            };

            let total_micros = transition
                .original_record
                .transition_time
                .as_micros(bpm as f64) as f32;

            transition.next_pos +=
                (transition.original_record.transition_interval * 1000.0) / total_micros;

            if transition.next_pos > 1.0 {
                transition.next_pos = 1.0;
            }

            // Calculate next_micros based on the target beat time, not by adding a fixed interval
            // This ensures the transition completes at the correct beat even when tempo changes
            let transition_progress =
                transition.original_record.transition_time.as_f64() * transition.next_pos as f64;
            let target_beat_f64 =
                transition.original_record.start_beat_time.as_f64() + transition_progress;
            let target_beat = BeatTime::from_parts(
                target_beat_f64.floor() as u32,
                (target_beat_f64.fract()) as f32,
            );
            let remaining_beats = target_beat - self.current_beat_time;
            transition.next_micros = self.current_micros + remaining_beats.as_micros(bpm as f64);
        }

        Some(res)
    }

    fn consume_beat(&mut self, next_beat_micros: u64) -> Option<MtxtOutputRecord> {
        let beat_time = self.next_beat_to_emit;
        self.next_beat_to_emit = self.next_beat_to_emit + BeatTime::from_parts(1, 0.0);

        self.current_micros = next_beat_micros;
        self.current_beat_time = beat_time;

        Some(MtxtOutputRecord::Beat {
            time: self.current_micros,
            beat: beat_time.as_f64() as u64,
        })
    }

    fn process(&mut self) -> Option<MtxtOutputRecord> {
        let (transition_idx, next_transition_micros) = match self
            .active_transitions
            .iter()
            .enumerate()
            .min_by_key(|(_, a)| a.next_micros)
        {
            Some((idx, t)) => (idx, t.next_micros),
            None => (usize::MAX, u64::MAX),
        };

        let next_record = self.records.get(self.next_record_idx);
        let next_record_micros = if let Some(record) = next_record {
            let next_beat_time = record.start_beat_time;
            let remaining_beats = next_beat_time - self.current_beat_time;
            let micros_to_next = remaining_beats.as_micros(self.current_bpm as f64);
            self.current_micros + micros_to_next
        } else {
            u64::MAX
        };

        if next_record_micros == u64::MAX && next_transition_micros == u64::MAX {
            return None;
        }

        // Calculate next beat time
        let next_beat_time = self.next_beat_to_emit;

        let next_beat_micros = if next_beat_time >= self.current_beat_time {
            let diff = next_beat_time - self.current_beat_time;
            self.current_micros + diff.as_micros(self.current_bpm as f64)
        } else {
            self.current_micros
        };

        // Determine which event is next
        let mut min_micros = next_transition_micros;
        let mut do_record = false;

        if next_record_micros <= min_micros {
            min_micros = next_record_micros;
            do_record = true;
        }

        if next_beat_micros <= min_micros {
            return self.consume_beat(next_beat_micros);
        }

        if do_record {
            return self.consume_record(next_record_micros);
        }

        self.consume_transition(transition_idx)
    }

    fn process_item(&mut self) -> Option<MtxtOutputRecord> {
        let res = self.process();
        if let Some(record) = res.as_ref() {
            match record {
                MtxtOutputRecord::Tempo { bpm, .. } => {
                    self.current_bpm = *bpm;
                    let key = record.get_param_key().unwrap();
                    self.last_values.insert(key, *bpm);
                }
                MtxtOutputRecord::ControlChange { value, .. } => {
                    let key = record.get_param_key().unwrap();
                    self.last_values.insert(key, *value);
                }
                _ => {}
            }
        }
        res
    }

    pub fn process_all(&mut self) -> Vec<MtxtOutputRecord> {
        let mut output = Vec::with_capacity(self.records.len() * 2);
        while let Some(record) = self.process_item() {
            output.push(record);
        }
        output
    }
}

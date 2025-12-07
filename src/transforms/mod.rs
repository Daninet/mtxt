pub mod apply;
pub mod exclude;
pub mod extract;
pub mod group;
pub mod include;
pub mod merge;
pub mod offset;
pub mod quantize;
pub mod sort;
pub mod transpose;

use crate::MtxtRecord;
use std::collections::HashSet;

pub struct TransformDescriptor {
    pub apply_directives: bool,
    pub extract_directives: bool,
    pub sort_by_time: bool,
    pub merge_notes: bool,
    pub quantize_grid: u32,
    pub quantize_swing: f32,
    pub quantize_humanize: f32,
    pub transpose_amount: i32,
    pub offset_amount: f32,
    pub include_channels: HashSet<u16>,
    pub exclude_channels: HashSet<u16>,
    pub group_channels: bool,
}

pub fn apply_transforms(
    records: &[MtxtRecord],
    transforms: &TransformDescriptor,
) -> Vec<MtxtRecord> {
    let mut current_records = records.to_vec();

    // order is important here

    if transforms.apply_directives {
        current_records = apply::transform(&current_records);
    }

    if !transforms.include_channels.is_empty() {
        current_records = include::transform(&current_records, &transforms.include_channels);
    }

    if !transforms.exclude_channels.is_empty() {
        current_records = exclude::transform(&current_records, &transforms.exclude_channels);
    }

    if transforms.transpose_amount != 0 {
        current_records = transpose::transform(&current_records, transforms.transpose_amount);
    }

    if transforms.offset_amount != 0.0 {
        current_records = offset::transform(&current_records, transforms.offset_amount);
    }

    if transforms.merge_notes {
        current_records = merge::transform(&current_records);
    }

    if transforms.quantize_grid > 0 {
        current_records = quantize::transform(
            &current_records,
            transforms.quantize_grid,
            transforms.quantize_swing,
            transforms.quantize_humanize,
        );
    }

    if transforms.sort_by_time {
        current_records = sort::transform(&current_records);
    }

    if transforms.group_channels {
        current_records = group::transform(&current_records);
    }

    if transforms.extract_directives {
        current_records = extract::transform(&current_records);
    }

    current_records
}

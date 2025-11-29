use crate::MtxtFile;
use crate::MtxtRecord;
use crate::parse_mtxt;

pub fn format_float32(value: f32) -> String {
    let trimmed_zeros = format!("{:.5}", value).trim_end_matches('0').to_string();

    if trimmed_zeros.ends_with('.') {
        trimmed_zeros + "0"
    } else {
        trimmed_zeros
    }
}

pub fn assert_eq_records(
    input: &str,
    transform: fn(&[MtxtRecord]) -> Vec<MtxtRecord>,
    expected: &str,
) {
    let input_parsed = parse_mtxt(input).expect("Failed to parse input");
    let expected_parsed = parse_mtxt(expected).expect("Failed to parse expected");
    let transformed = transform(&input_parsed.records);
    assert_eq!(
        transformed.len(),
        expected_parsed.records.len(),
        "length mismatch {} != {} , output={:?}",
        transformed.len(),
        expected_parsed.records.len(),
        MtxtFile::from_records(transformed)
            .to_string()
            .split("\n")
            .collect::<Vec<_>>(),
    );
    for (record, expected) in transformed.iter().zip(expected_parsed.records.iter()) {
        assert_eq!(record, expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(format_float32(1.234567), "1.23457");
        assert_eq!(format_float32(1.234547), "1.23455");
        assert_eq!(format_float32(1.23), "1.23");
        assert_eq!(format_float32(1.2000), "1.2");
        assert_eq!(format_float32(-5.0), "-5.0");
        assert_eq!(format_float32(0.0), "0.0");
        assert_eq!(format_float32(-0.0), "-0.0");
        assert_eq!(format_float32(0.0023), "0.0023");
        assert_eq!(format_float32(123456789123.456), "123456790528.0");
    }
}

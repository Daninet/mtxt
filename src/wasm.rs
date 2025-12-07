use crate::midi;
use crate::parser::parse_mtxt;
use crate::transforms::TransformDescriptor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmTransformDescriptor {
    pub apply_directives: bool,
    pub extract_directives: bool,
    pub sort_by_time: bool,
    pub merge_notes: bool,
    pub quantize_grid: u32,
    pub quantize_swing: f32,
    pub quantize_humanize: f32,
    pub transpose_amount: i32,
    pub offset_amount: f32,
    include_channels: Vec<u16>,
    exclude_channels: Vec<u16>,
    pub group_channels: bool,
}

#[wasm_bindgen]
impl WasmTransformDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            apply_directives: false,
            extract_directives: false,
            sort_by_time: false,
            merge_notes: false,
            quantize_grid: 0,
            quantize_swing: 0.0,
            quantize_humanize: 0.0,
            transpose_amount: 0,
            offset_amount: 0.0,
            include_channels: Vec::new(),
            exclude_channels: Vec::new(),
            group_channels: false,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn include_channels(&self) -> Vec<u16> {
        self.include_channels.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_include_channels(&mut self, channels: Vec<u16>) {
        self.include_channels = channels;
    }

    #[wasm_bindgen(getter)]
    pub fn exclude_channels(&self) -> Vec<u16> {
        self.exclude_channels.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_exclude_channels(&mut self, channels: Vec<u16>) {
        self.exclude_channels = channels;
    }
}

impl From<&WasmTransformDescriptor> for TransformDescriptor {
    fn from(w: &WasmTransformDescriptor) -> Self {
        Self {
            apply_directives: w.apply_directives,
            extract_directives: w.extract_directives,
            sort_by_time: w.sort_by_time,
            merge_notes: w.merge_notes,
            quantize_grid: w.quantize_grid,
            quantize_swing: w.quantize_swing,
            quantize_humanize: w.quantize_humanize,
            transpose_amount: w.transpose_amount,
            offset_amount: w.offset_amount,
            include_channels: w.include_channels.iter().cloned().collect(),
            exclude_channels: w.exclude_channels.iter().cloned().collect(),
            group_channels: w.group_channels,
        }
    }
}

#[wasm_bindgen]
pub fn midi_to_mtxt(midi_bytes: &[u8], format_padding: bool) -> Result<String, JsError> {
    let mtxt_file =
        midi::convert_midi_to_mtxt(midi_bytes).map_err(|e| JsError::new(&e.to_string()))?;

    let timestamp_width = if format_padding {
        Some(mtxt_file.calculate_auto_timestamp_width())
    } else {
        None
    };

    Ok(format!(
        "{}",
        mtxt_file.display_with_formatting(timestamp_width)
    ))
}

#[wasm_bindgen]
pub fn mtxt_to_midi(mtxt_content: &str) -> Result<Vec<u8>, JsError> {
    let mtxt_file = parse_mtxt(mtxt_content).map_err(|e| JsError::new(&e.to_string()))?;

    let midi_bytes =
        midi::convert_mtxt_to_midi(&mtxt_file).map_err(|e| JsError::new(&e.to_string()))?;

    Ok(midi_bytes)
}

#[wasm_bindgen]
pub fn apply_transforms(
    mtxt_content: &str,
    descriptor: &WasmTransformDescriptor,
    format_padding: bool,
) -> Result<String, JsError> {
    let mut mtxt_file = parse_mtxt(mtxt_content).map_err(|e| JsError::new(&e.to_string()))?;

    let transforms: TransformDescriptor = descriptor.into();

    mtxt_file.records = crate::transforms::apply_transforms(&mtxt_file.records, &transforms);

    let timestamp_width = if format_padding {
        Some(mtxt_file.calculate_auto_timestamp_width())
    } else {
        None
    };

    Ok(format!(
        "{}",
        mtxt_file.display_with_formatting(timestamp_width)
    ))
}

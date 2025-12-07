import init, { 
    midi_to_mtxt, 
    mtxt_to_midi, 
    apply_transforms, 
    WasmTransformDescriptor,
    init_panic_hook 
} from './pkg/mtxt.js';

const statusEl = document.getElementById('status');
const editor = document.getElementById('editor');
const fileInput = document.getElementById('midi-input');

async function run() {
    try {
        await init();
        init_panic_hook();
        statusEl.textContent = 'Ready';
        console.log('WASM loaded');
    } catch (e) {
        console.error(e);
        statusEl.textContent = 'Error loading WASM';
    }
}

run();

// File Operations
document.getElementById('open-btn').addEventListener('click', () => fileInput.click());

fileInput.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    statusEl.textContent = 'Reading MIDI...';
    try {
        const arrayBuffer = await file.arrayBuffer();
        const bytes = new Uint8Array(arrayBuffer);
        const formatPadding = document.getElementById('format_padding').checked;
        const mtxt = midi_to_mtxt(bytes, formatPadding);
        editor.value = mtxt;
        statusEl.textContent = `Loaded ${file.name}`;
    } catch (err) {
        console.error(err);
        alert('Failed to convert MIDI to MTXT: ' + err);
        statusEl.textContent = 'Error';
    }
    fileInput.value = '';
});

document.getElementById('save-btn').addEventListener('click', () => {
    const content = editor.value;
    if (!content.trim()) return;

    statusEl.textContent = 'Converting to MIDI...';
    try {
        const midiBytes = mtxt_to_midi(content);
        downloadBlob(midiBytes, 'output.mid', 'audio/midi');
        statusEl.textContent = 'Saved MIDI';
    } catch (err) {
        console.error(err);
        alert('Failed to convert MTXT to MIDI: ' + err);
        statusEl.textContent = 'Error';
    }
});

// Transform Logic
const applyTransforms = (config = {}) => {
    const content = editor.value;
    if (!content.trim()) return;

    const {
        apply_directives = false,
        extract_directives = false,
        sort_by_time = false,
        merge_notes = false,
        group_channels = false,
        transform_type = null,
        transform_value = null
    } = config;

    statusEl.textContent = 'Applying transforms...';
    try {
        const descriptor = new WasmTransformDescriptor();
        
        // Set Boolean Flags
        descriptor.apply_directives = apply_directives;
        descriptor.extract_directives = extract_directives;
        descriptor.sort_by_time = sort_by_time;
        descriptor.merge_notes = merge_notes;
        descriptor.group_channels = group_channels;
        
        // Set Single Parameter
        if (transform_type && transform_value !== null) {
            const valStr = String(transform_value);
            
            if (transform_type === 'quantize_grid') {
                 descriptor.quantize_grid = parseInt(valStr) || 0;
            } else if (transform_type === 'quantize_swing') {
                 descriptor.quantize_swing = parseFloat(valStr) || 0;
            } else if (transform_type === 'quantize_humanize') {
                 descriptor.quantize_humanize = parseFloat(valStr) || 0;
            } else if (transform_type === 'transpose_amount') {
                 descriptor.transpose_amount = parseInt(valStr) || 0;
            } else if (transform_type === 'offset_amount') {
                 descriptor.offset_amount = parseFloat(valStr) || 0;
            } else if (transform_type === 'include_channels' || transform_type === 'exclude_channels') {
                const parseChannels = (str) => {
                    if (!str.trim()) return new Uint16Array([]);
                    const nums = str.split(',')
                        .map(s => parseInt(s.trim()))
                        .filter(n => !isNaN(n));
                    return new Uint16Array(nums);
                };
                const channels = parseChannels(valStr);
                if (transform_type === 'include_channels') {
                    descriptor.include_channels = channels;
                } else {
                    descriptor.exclude_channels = channels;
                }
            }
        }

        const formatPadding = document.getElementById('format_padding').checked;
        const newContent = apply_transforms(content, descriptor, formatPadding);
        editor.value = newContent;
        statusEl.textContent = 'Transforms applied';
        
        descriptor.free();
    } catch (err) {
        console.error(err);
        alert('Failed to apply transforms: ' + err);
        statusEl.textContent = 'Error';
    }
};

// Event Listeners for Quick Actions
document.getElementById('btn-apply-directives').addEventListener('click', () => 
    applyTransforms({ apply_directives: true }));
    
document.getElementById('btn-extract-directives').addEventListener('click', () => 
    applyTransforms({ extract_directives: true }));
    
document.getElementById('btn-sort').addEventListener('click', () => 
    applyTransforms({ sort_by_time: true }));
    
document.getElementById('btn-merge').addEventListener('click', () => 
    applyTransforms({ merge_notes: true }));
    
document.getElementById('btn-group').addEventListener('click', () => 
    applyTransforms({ group_channels: true }));

// Event Listener for Single Parameter
document.getElementById('btn-apply-single').addEventListener('click', () => {
    const type = document.getElementById('transform_type').value;
    const value = document.getElementById('transform_value').value;
    applyTransforms({ transform_type: type, transform_value: value });
    document.getElementById('transform_value').value = '';
});

// Dynamic Placeholder Logic
const transformTypeSelect = document.getElementById('transform_type');
const transformValueInput = document.getElementById('transform_value');

const placeholders = {
    'quantize_grid': '0=off, 4=1/4 note, 8=1/8 note',
    'quantize_swing': 'Amount (0.0 - 1.0)',
    'quantize_humanize': 'Amount (0.0 - 1.0)',
    'transpose_amount': 'Semitones (e.g. 12, -5)',
    'offset_amount': 'Beats (e.g. 0.5, -1.0)',
    'include_channels': 'Channel numbers (e.g. 1, 2)',
    'exclude_channels': 'Channel numbers (e.g. 10)'
};

const updatePlaceholder = () => {
    const type = transformTypeSelect.value;
    transformValueInput.placeholder = placeholders[type] || 'Value';
    transformValueInput.value = ''; // Clear value on change
};

transformTypeSelect.addEventListener('change', updatePlaceholder);

// Initialize placeholder
updatePlaceholder();

// Event Listener for Format Timestamps Toggle
document.getElementById('format_padding').addEventListener('change', () => 
    applyTransforms());

function downloadBlob(data, fileName, mimeType) {
    const blob = new Blob([data], { type: mimeType });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    a.style.display = 'none';
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
}

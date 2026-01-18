# MTXT - Music Text Format

MTXT is a human-editable, text-based format for representing musical performance data. It stores events with precise timing, pitch, and expression values in a simple line-based structure that's easy to edit without requiring specialized tools.

The format is designed for cases where exact performance details matter - arbitrary timings, micro-tuning, dynamic changes, and other expressive parameters. MTXT avoids binary encoding, making it suitable for manual and AI-assisted editing.

## Features
- **Beat-based**: Events are placed on fractional beats using simple decimal notation (e.g., in a 4/4 time signature, 1 beat = 1 quarter note, so 3.25 represents 3 quarter notes plus 1 sixteenth note).
- **One event per line**: Easy to search and modify with any text editor.
- **Human-friendly**: Use of musical note names (C4, D#3, etc.) and custom note aliases (e.g. `kick` or `Cmaj7`). Hand-crafted files are practical and expressive.
- **Transitions**: Built-in support for gliding continuous parameters (CC, tempo) with customizable curves and timing.
- **Real-time ready**: Supports streaming musical events and real-time data transfer with transitions and flexible timing.
- **Microtonal support**: Built-in cents notation for notes (e.g. `C4+50`, `D4-25`) and global tuning commands for alternate tuning systems and just intonation.  
- **Flexible organization**: Events can be written in any order in the file, with the parser handling chronological sorting.
- **MIDI compatible**: Reference implementation includes MIDI to MTXT and MTXT to MIDI conversion.
- **LLM compatible**: Can be easily generated and manipulated by language models.
- **Future-proof**: Supports up to 65535 channels, arbitrary CC parameters with custom string keys and custom metadata.  

## Quick Example
```
mtxt 1.0
meta global title Sunrise Melody
meta global author Jane Composer

// Define aliases for drums and chords
alias kick C1
alias Cmaj7 C4,E4,G4,B4

// Global tempo and time signature
0.0 tempo 100
0.0 timesig 4/4

// Set defaults for channel, duration, and velocity
ch=0
dur=1.0
vel=0.8

// Set voice. "John's bright grand" has precedence, but it falls back to a more generic "piano" if not found.
0.0 voice piano, John's bright grand

// Start silently
0.0 cc volume 0.0

// Fade in volume over 3.0 beats, ending at beat 4.0
4.0 cc volume 1.0 transition_time=3.0 transition_curve=0.5

// Play melody (uses default duration and velocity from above, unless overridden)
0.0 note C4
2.0 note G4 vel=0.5
// Notes can be put in arbitrary order, the parser will sort them
1.0 note E4 
2.0 note G4 vel=0.5

// Chords can also be played (defined above as an alias for C4,E4,G4,B4 notes)
1.0 note Cmaj7 dur=2.0 vel=0.2

// Tempo ramp
8.0 tempo 120 transition_time=4.0

// Microtonal note (12TET equal temperament C4 + 50 cents)
3.0 note C4+50
```

## Rust Library and CLI Tool

This repository includes a reference implementation in Rust that provides:

- **Library (`mtxt`)**: Rust crate for parsing and writing MTXT files, with MIDI conversion features.
- **CLI tool**: Command-line utility for converting between MIDI and MTXT formats with built-in transforms. Builds can be downloaded from [releases](https://github.com/Daninet/mtxt/releases).


### Basic Usage

```bash
mtxt input.mid output.mtxt # MIDI to MTXT
mtxt input.mtxt output.mid # MTXT to MIDI
mtxt input.mtxt output.mtxt --sort # transform MTXT file and sort events by time
```

### Transform Options

The CLI supports various transforms that can be applied during conversion:

**Musical Transforms:**
- `--transpose <SEMITONES>` - Transpose all notes by semitones (e.g., `--transpose +2` or `--transpose -12`)
- `-q, --quantize <GRID>` - Quantize timing to a grid (e.g., `4` for quarter notes, `16` for 16th notes)
- `--offset <BEATS>` - Offset all events by beats (e.g., `--offset 1.5`, `--offset -0.5`). Events shifted to negative times are removed.
- `--swing <AMOUNT>` - Apply swing feel (0.0 to 1.0)
- `--humanize <AMOUNT>` - Add timing randomization for humanization (0.0 to 1.0)

**Channel Filtering:**
- `--include-channels <CHANNELS>` - Include only specific channels (comma-separated, e.g., `1,2,10`)
- `--exclude-channels <CHANNELS>` - Exclude specific channels (comma-separated, e.g., `1,2,10`)

**File Organization:**
- `--apply-directives` - Apply global directives to events (inline parameters)
- `--extract-directives` - Extract common inline parameters into global directives
- `--merge-notes` - Merge note on / off pairs into note shorthand events with durations
- `--group-channels` - Group events by channel
- `--sort` - Sort events by time
- `--indent` - Enable timestamp padding

---

## MTXT Specification

## Versioning

- First line must declare version:
  ```
  mtxt 1.0
  ```

## Structure

- A file consists of:
  1. Version line
  2. Global metadata (optional)
  3. Events (can be in any timestamp order)

## Timing

- All times are in beats specified as decimal numbers. e.g. in a 4/4 time signature, 1 beat = 1 quarter note, so 3.25 represents 3 quarter notes plus 1 sixteenth note.
- This allows changing the tempo and time signature without affecting the timing of events.
- Events may appear in **any order** in the file; the parser will sort them before playback
- Precision is limited to 5 decimal places (5 microseconds at 120 BPM).

## Timing with fractional expressions

- Complex timing expressions are also supported (e.g., `1.0+1/4`).
- Supported expression operators: `+`, `-`, `*`.
- **Fractions**: Specified as `integer/integer` (e.g., `1/4`, `3/2`).
- **Multiplication**: Operands must be explicit fractions (e.g., `1/2*2/3`).
- **Order of operations**: Fractions are evaluated first, then multiplications, then additions and subtractions.
- No spaces are allowed in the expression.

## Commands

### version (mtxt)
```
mtxt <semver>
```
- Declares the file format version in the major.minor format (e.g., `mtxt 1.0`).
- Must be the first non-comment line.

### meta
```
// global meta (applies to the entire file and all channels)
meta global <type> <value>
// channel meta (applies to a single channel), starting from the specified time
[<time>] meta [ch=<0..65535>] <type> <value>
```
- Adds metadata (e.g., `title`, `author`, `copyright`, `trackname`, custom types).
- Value extends from the type to the end of the line (or until inline comment).
- Channel is optional, if not specified it inherits the channel from the previous `ch` command. 
- Time is optional, defaults to 0.0.
- See [Standard Meta Types](#standard-meta-types) for a list of standard types.
- Newline characters in <value> need to be escaped to avoid breaking the syntax.

#### Standard Meta Types

| Type           | Description      | Example                                   |
| -------------- | ---------------- | ----------------------------------------- |
| `title`        | Song title       | `meta global title My Song`               |
| `author`       | Composer/author  | `meta global author John Doe`             |
| `copyright`    | Copyright notice | `meta global copyright © 2024 Music Corp` |
| `composer`     | Composer name    | `meta global composer John Doe`           |
| `name`         | Channel name     | `meta ch=1 name Lead Guitar`              |
| `instrument`   | Instrument name  | `meta ch=2 instrument Steinway Grand`     |
| `smpte`        | SMPTE offset     | `meta global smpte 00:00:00:00`           |
| `keysignature` | Key signature    | `4.0 meta ch=3 keysignature C major`      |
| `text`         | General text     | `meta text Verse 1`                       |
| `lyric`        | Lyrics           | `5.0 meta lyric Hello world`              |
| `marker`       | Marker/cue point | `6.0 meta marker Chorus`                  |
| `cue`          | Cue point        | `7.0 meta cue Solo begins`                |
| `program`      | Program name     | `meta global program Piano Ballad`        |
| `device`       | Device name      | `meta global device OpenPiano 1000`       |
| `key`          | Key signature    | `meta global key C major`                 |
| `date`         | Creation date    | `meta global date 2024-01-01`             |
| `genre`        | Musical genre    | `meta global genre Rock`                  |
| `album`        | Album name       | `meta global album Greatest Hits`         |
| `url`          | Related URL      | `meta global url https://example.com`     |
| `artist`       | Performer name   | `meta global artist The Band`             |
| `license`      | Usage license    | `meta global license CC-BY-4.0`           |
| `generator`    | Software tool    | `meta global generator MySequencer v1.0`  |


### ch (channel directive)
```
ch=<0..65535>
```
- Sets the default MIDI channel for subsequent events.
- Inline `ch=<N>` on events overrides the default for that event only.
- Required before channel-dependent events that omit inline `ch`.

### alias (note naming)
```
alias <name> <value>
```
- Defines a named alias for a note pitch or a chord.
- `<name>`: Alphanumeric identifier (e.g., `snare`, `Cmaj7`).
- `<value>`: Target note(s), comma-separated if multiple.
- No timestamp. Applies to all subsequent events in the file until overridden.
- Name is case-insensitive.
- Example:
  ```
  alias snare C2
  alias Cmaj7 C4,E4,G4,B4  // chord alias
  0.0 note snare
  1.0 note Cmaj7          // plays all 4 notes
  ```

### vel (default note-on velocity)
```
vel=<0.0..1.0>
```
- Sets the default note-on velocity.
- Inline `vel=<N>` on `note`/`on` overrides for that event.

### offvel (default note-off velocity)
```
offvel=<0.0..1.0>
```
- Sets the default note-off velocity.
- Inline `offvel=<N>` on `note`/`off` overrides for that event.
- Defaults to `1.0` if not set.
 
### dur (default note duration)
```
dur=<float>
```
- Sets the default note duration in beats.
- Inline `dur=<N>` on `note` overrides for that event.
- Defaults to `1.0` if not set.
 
### transition settings
```
transition_curve=<float>
transition_interval=<float>
```
- Sets default transition parameters for `cc` and `tempo`.
- `transition_time` must be specified per event (default `0.0`).
- See **Transitions** section for details.
- Defaults: `curve=0.0`, `interval=1.0`.

### note (shorthand)
```
<time> note <NOTE> [dur=<float>] [vel=<0..1>] [offvel=<0..1>] [ch=<0..65535>]
```
- Emits note-on at `<time>` and note-off at `<time + dur>`.
- Uses defaults from `dur`, `vel`, `offvel`, and `ch` unless overridden.
- `<time>` is absolute beat `BEAT.SUB` (0-based). Example: `3.5`.
- `<NOTE>` can be a standard note name or an `alias`.
- Standard names support `C..B` with `#`/`b`, octave required (e.g., `C4`).
  - Allowed: `C, C#, Db, D, D#, Eb, E, F, F#, Gb, G, G#, Ab, A, A#, Bb, B`
  - **Case insensitive**: Both uppercase and lowercase are accepted (`C4`, `c4`, `Bb2`, `bb2`, `F#3`, `f#3`)
  - Double accidentals **not allowed**.
  - Microtonal: `+N`/`-N` cents (range `-99..+99`), applied via pitch bend. Examples: `C4+50` (50 cents sharp), `D4-25` (25 cents flat), `bb2+10.5` (10.5 cents sharp). Positive values require `+`.

### on (note-on)
```
<time> on <NOTE> [vel=<0..1>] [ch=<0..65535>]
```
- Emits a note-on only; useful for streaming.
- Uses default `vel` and `ch` unless overridden.

### off (note-off)
```
<time> off <NOTE> [offvel=<0..1>] [ch=<0..65535>]
```
- Emits a note-off only; useful for streaming.
- Uses default `offvel` and `ch` unless overridden.

### cc (control change)
```
<time> cc [note] <controller> <value> [ch=<0..65535>] [transition_curve=<float>] [transition_time=<float>] [transition_interval=<float>]
```
- Sends a control change. `<controller>` identifies the parameter.
- `<value>` is `[0.0..1.0]`. Uses default `ch` unless overridden.
- Uses global transition defaults unless overridden inline.
- Optional `note` to apply CC to a specific note. If a note is specified, it applies to that note only.
- When a note is not specified, it applies to all notes within a channel. 
- Arbitrary string keys can be used for custom parameters (e.g., `cc my_param 0.5`).

#### Standard CC Names

| Name                | Range         | Description                                                                     |
| ------------------- | ------------- | ------------------------------------------------------------------------------- |
| `pitch`             | `-12.0..12.0` | Pitch bend in semitones. 0=none, 1 = one semitone up, `-0.005` = half cent down |
| `aftertouch`        | `0.0..1.0`    | Channel or Polyphonic Aftertouch                                                |
| `vibrato`           | `0.0..1.0`    | Vibrato depth (Modulation Wheel)                                                |
| `vibrato_rate`      | `0.0..1024.0` | Vibrato rate in Hz                                                              |
| `breath`            | `0.0..1.0`    | Breath controller pressure                                                      |
| `foot`              | `0.0..1.0`    | Foot controller pedal position                                                  |
| `portamento`        | `0.0..1.0`    | Portamento glide time                                                           |
| `volume`            | `0.0..1.0`    | Channel volume level                                                            |
| `balance`           | `-1.0..1.0`   | Stereo balance (left < 0, right > 0)                                            |
| `pan`               | `-1.0..1.0`   | Stereo panning position (left < 0, right > 0)                                   |
| `expression`        | `0.0..1.0`    | Expression (secondary volume, relative to main)                                 |
| `sustain`           | `0.0..1.0`    | Sustain pedal (damper) on/off (> 0.5 is on)                                     |
| `portamento_switch` | `0.0..1.0`    | Portamento on/off switch (> 0.5 is on)                                          |
| `sostenuto`         | `0.0..1.0`    | Sostenuto pedal on/off (> 0.5 is on)                                            |
| `soft`              | `0.0..1.0`    | Soft pedal (una corda) on/off (> 0.5 is on)                                     |
| `legato`            | `0.0..1.0`    | Legato footswitch on/off (> 0.5 is on)                                          |
| `sound_variation`   | `0.0..1.0`    | Sound variation                                                                 |
| `timbre`            | `0.0..1.0`    | Timbre/harmonic intensity                                                       |
| `resonance`         | `0.0..1.0`    | Resonance/Harmonic Content                                                      |
| `attack`            | `0.0..1.0`    | Attack time                                                                     |
| `decay`             | `0.0..1.0`    | Decay time                                                                      |
| `hold`              | `0.0..1.0`    | Hold time                                                                       |
| `sustain_level`     | `0.0..1.0`    | Sustain level (envelope)                                                        |
| `release`           | `0.0..1.0`    | Release time                                                                    |
| `cutoff`            | `0.0..1.0`    | Filter cutoff frequency (Brightness)                                            |
| `reverb`            | `0.0..1.0`    | Reverb send level                                                               |
| `tremolo`           | `0.0..1.0`    | Tremolo depth                                                                   |
| `tremolo_rate`      | `0.0..1024.0` | Tremolo rate in Hz                                                              |
| `chorus`            | `0.0..1.0`    | Chorus send level                                                               |
| `detune`            | `0.0..1.0`    | Detune depth                                                                    |
| `phaser`            | `0.0..1.0`    | Phaser depth                                                                    |
| `distortion`        | `0.0..1.0`    | Distortion/Drive amount                                                         |
| `compression`       | `0.0..1.0`    | Compression amount                                                              |
| `local_control`     | `0.0..1.0`    | Local control on/off (> 0.5 is on)                                              |
| `polyphony`         | `1.0..1024.0` | Polyphony count (rounded to int). 1=mono                                        |

### voice (instrument selection)
```
<time> voice [ch=<0..65535>] <voice_list>
```
- Sets the instrument voice for the channel.
- `<voice_list>` is a comma-separated list of voice names (e.g., `piano, acoustic piano, john's super piano`).
- The synthesizer should use the **last** voice in the list that it supports.
- It is recommended to use a standard voice from `instruments.md` as the first item for compatibility.


### tempo
```
<time> tempo <bpm> [transition_curve=<float>] [transition_time=<float>] [transition_interval=<float>]
```
- Sets tempo in BPM at `<time>`.
- Uses global transition defaults unless overridden inline.
- By default, tempo is at 120 BPM at the start of the file.

### timesig
```
<time> timesig <NUM>/<DEN>
```
- Sets time signature; affects beat interpretation after `<time>`.
- By default, time signature is 4/4 at the start of the file.

### tuning
```
<time> tuning <target> <cents>
```
- By default, notes are defined with equal temperament tuning (12TET).
- Sets the global tuning offset for a note or pitch class.
- `<target>` can be a pitch class (e.g., `C`, `F#`) or a specific note (e.g., `C4`).
- `<cents>` is in range `[-100.0..+100.0]`. Positive values must use `+`.
- Tuning is additive: `Final Pitch = Standard Pitch + Tuning + Note Offset`.
- Changes are instantaneous (no interpolation).
- Specific note tuning overrides pitch class tuning.
- Example:
  ```
  0.0 tuning E -13.7   // Flatten all Es
  0.0 tuning G +3.5    // Sharpen all Gs
  0.0 tuning E4 0.0    // Exception: E4 is standard tuning (overrides previous line)
  ```

### reset
```
<time> reset [target]
```
- Resets state variables to defaults.
- `<target>` options:
  - `all` (default): All notes off, all controllers reset, tuning cleared on all channels.
  - `ch=<N>`: Resets controllers and turns off notes on specific channel.
  - `tuning`: Clears all global tuning definitions.
- Example:
  ```
  10.0 reset all
  12.0 reset ch=1
  14.0 reset tuning
  ```

### sysex
```
<time> sysex <hex-bytes>
```
- Sends raw SysEx bytes (space-separated hex, including `F0`/`F7` as needed).
- Example: `12.0 sysex F0 7E 7F 09 01 F7`

### Comments
```
// full-line comment
<command> // inline comment
```
- Everything after `//` is ignored by the parser (except for `://` in URLs).

## Transitions

Use transitions to glide a continuous parameter to a target value by a specific beat, with a chosen feel.

- Supported on: `cc`, `tempo`.
- Fields:
  - `transition_curve=<alpha>` controls the feel of the glide:
    - `0.0` (linear): steady change from start to finish (default)
    - `> 0` (gentle start → speeds up): musical "ease-in", swells late.
    - `< 0` (fast start → settles): musical "ease-out", arrives smoothly.
  - `transition_time=<duration>` (`τ`) is the glide length in beats. Defaults to `0.0` (instant jump). The change begins at `T − τ` and reaches the target at command time `T`.
  - `transition_interval=<duration>` is the minimum time between each value update in milliseconds. Defaults to `1.0` (as fast as possible).

Examples:
- `0.0 cc pitch 0.0` — pitch is `0.0` at `0.0`.
- `1.0 cc pitch 0.5 transition_time=0.2` — pitch glides to `0.5` value between beats `0.8` and `1.0`.
- `5.0 cc pitch 0.95 transition_curve=0.5 transition_time=1.5` — starts at `3.5`, accelerates toward `0.95` near `5.0`.
- `7.0 cc volume 0.2 transition_curve=-0.4 transition_time=2.0` — begins at `5.0`, moves quickly then coasts into `0.2` at `7.0`.

Curve definition:
- `value(t) = V0 + (V1 − V0) * ( s + max(α,0) * (s^4 − s) − max(−α,0) * ((1 − (1 − s)^4) − s) )`
- `s = (t − (T − τ)) / τ`; if `τ = 0`, the change is instant at `T`.

Notes:
- Each transition needs a defined value before its end time `T` to establish the start (`V0` at `T − τ`). If no prior value exists, this is an error.
- Overlapping transitions on the same parameter/channel: the new transition immediately aborts the previous at the current value and takes over. When segments conflict, the one with the later end beat (`T`) has precedence.


## Common note lengths

|                       | Fraction | Expanded          | Beats     |
|-----------------------|----------|-------------------|-----------|
| Whole                 | `4/1`    |                   | 4.0       |
| Half                  | `2/1`    |                   | 2.0       |
| Quarter               | `1/1`    |                   | 1.0       |
| Eighth                | `1/2`    |                   | 0.5       |
| Sixteenth             | `1/4`    |                   | 0.25      |
| Thirty-second         | `1/8`    |                   | 0.125     |
| Dotted half           | `3/1`    | `2 + 1`           | 3.0       |
| Dotted quarter        | `3/2`    | `1 + 1/2`         | 1.5       |
| Dotted eighth         | `3/4`    | `1/2 + 1/4`       | 0.75      |
| Dotted sixteenth      | `3/8`    | `1/4 + 1/8`       | 0.375     |
| Double-dotted quarter | `7/4`    | `1 + 1/2 + 1/4`   | 1.75      |
| Double-dotted eighth  | `7/8`    | `1/2 + 1/4 + 1/8` | 0.875     |
| Quarter triplet       | `2/3`    | `1 * 2/3`         | 0.666667  |
| Eighth triplet        | `1/3`    | `1/2 * 2/3`       | 0.333333  |
| Sixteenth triplet     | `1/6`    | `1/4 * 2/3`       | 0.166667  |
| Quarter quintuplet    | `4/5`    | `1 * 4/5`         | 0.8       |
| Eighth quintuplet     | `2/5`    | `1/2 * 4/5`       | 0.4       |
| Sixteenth quintuplet  | `1/5`    | `1/4 * 4/5`       | 0.2       |
| Quarter septuplet     | `4/7`    | `1 * 4/7`         | 0.571429  |
| Eighth septuplet      | `2/7`    | `1/2 * 4/7`       | 0.285714  |
| Sixteenth septuplet   | `1/7`    | `1/4 * 4/7`       | 0.142857  |

## License

Copyright © 2025 Dani Biró

MTXT specification and reference implementation are dual-licensed under the Apache-2.0 license or the MIT license, at your option.


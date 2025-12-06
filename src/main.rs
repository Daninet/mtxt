use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::path::Path;

#[cfg(feature = "midi")]
use mtxt::midi;

#[derive(Debug, PartialEq)]
enum FileFormat {
    Midi,
    Mtxt,
}

fn detect_file_format(file_path: &str) -> Result<FileFormat> {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", file_path))?;

    match extension.to_lowercase().as_str() {
        "mid" => Ok(FileFormat::Midi),
        "midi" => Ok(FileFormat::Midi),
        "smf" => Ok(FileFormat::Midi),
        "mtxt" => Ok(FileFormat::Mtxt),
        _ => Err(anyhow::anyhow!(
            "Unsupported file extension: .{}",
            extension
        )),
    }
}

fn main() -> Result<()> {
    println!("MTXT Converter v{}", env!("CARGO_PKG_VERSION"));
    println!("");

    let matches = Command::new("mtxt")
        .version(env!("CARGO_PKG_VERSION"))
        .about("MTXT converter")
        .arg(
            Arg::new("input")
                .help("Input file (.mid or .mtxt)")
                .required(true)
                .value_name("INPUT_FILE")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .help("Output file (.mid or .mtxt)")
                .required(true)
                .value_name("OUTPUT_FILE")
                .index(2),
        )
        .arg(
            Arg::new("verbose")
                .help("Enable verbose output")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("transpose")
                .help("Transpose by semitones (e.g. +1, -12)")
                .long("transpose")
                .allow_hyphen_values(true)
                .value_name("SEMITONES")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("offset")
                .help("Offset all events by beats (e.g. 1.5, -0.5)")
                .long("offset")
                .allow_hyphen_values(true)
                .value_name("BEATS")
                .value_parser(clap::value_parser!(f32)),
        )
        .arg(
            Arg::new("include-channels")
                .help("Include only specific channels (comma-separated, e.g. 1,2,10)")
                .long("include-channels")
                .value_name("CHANNELS")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("exclude-channels")
                .help("Exclude specific channels (comma-separated, e.g. 1,2,10)")
                .long("exclude-channels")
                .value_name("CHANNELS")
                .value_delimiter(',')
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("apply-directives")
                .help("Apply directives to events")
                .long("apply-directives")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("sort")
                .help("Sort events by time (respecting directives)")
                .long("sort")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("extract-directives")
                .help("Extract common inline parameters into global directives")
                .long("extract-directives")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("merge-notes")
                .help("Merge note on / off pairs into note shorthand events with durations")
                .long("merge-notes")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quantize")
                .help("Quantize grid (e.g. 4 for quarter notes, 16 for 16th notes)")
                .long("quantize")
                .short('q')
                .value_name("GRID")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("swing")
                .help("Swing amount (0.0 to 1.0)")
                .long("swing")
                .value_name("AMOUNT")
                .value_parser(clap::value_parser!(f32)),
        )
        .arg(
            Arg::new("humanize")
                .help("Humanize amount (0.0 to 1.0)")
                .long("humanize")
                .value_name("AMOUNT")
                .value_parser(clap::value_parser!(f32)),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let verbose = matches.get_flag("verbose");
    let apply_directives = matches.get_flag("apply-directives");
    let sort_by_time = matches.get_flag("sort");
    let merge_notes = matches.get_flag("merge-notes");
    let extract_directives = matches.get_flag("extract-directives");

    let transpose_amount = matches.get_one::<i32>("transpose").copied().unwrap_or(0);
    let offset_amount = matches.get_one::<f32>("offset").copied().unwrap_or(0.0);
    let quantize_grid = matches.get_one::<u32>("quantize").copied().unwrap_or(0);
    let quantize_swing = matches.get_one::<f32>("swing").copied().unwrap_or(0.0);
    let quantize_humanize = matches.get_one::<f32>("humanize").copied().unwrap_or(0.0);

    let include_channels: std::collections::HashSet<u16> = matches
        .get_many::<u16>("include-channels")
        .unwrap_or_default()
        .copied()
        .collect();

    let exclude_channels: std::collections::HashSet<u16> = matches
        .get_many::<u16>("exclude-channels")
        .unwrap_or_default()
        .copied()
        .collect();

    let transforms = mtxt::transforms::TransformDescriptor {
        apply_directives,
        extract_directives,
        sort_by_time,
        merge_notes,
        quantize_grid,
        quantize_swing,
        quantize_humanize,
        transpose_amount,
        offset_amount,
        include_channels,
        exclude_channels,
    };

    let input_format = detect_file_format(input_file)
        .with_context(|| format!("Failed to detect input file format: {}", input_file))?;

    let output_format = detect_file_format(output_file)
        .with_context(|| format!("Failed to detect output file format: {}", output_file))?;

    if verbose {
        println!(
            "Input format: {:?}, Output format: {:?}",
            input_format, output_format
        );
    }

    let mut mtxt_file = match input_format {
        FileFormat::Midi => {
            #[cfg(feature = "midi")]
            {
                if verbose {
                    println!("Reading MIDI file: {}", input_file);
                }
                midi::convert_midi_to_mtxt(input_file, verbose)
                    .context("Failed to convert MIDI to MTXT")?
            }
            #[cfg(not(feature = "midi"))]
            {
                anyhow::bail!("MIDI support is not enabled. Compile with --features midi");
            }
        }
        FileFormat::Mtxt => {
            if verbose {
                println!("Reading MTXT file: {}", input_file);
            }
            let content = std::fs::read_to_string(input_file)
                .with_context(|| format!("Failed to read input file: {}", input_file))?;
            mtxt::parse_mtxt(&content)
                .with_context(|| format!("Failed to parse MTXT file: {}", input_file))?
        }
    };

    if verbose {
        println!("Applying transforms...");
    }
    mtxt_file.records = mtxt::transforms::apply_transforms(&mtxt_file.records, &transforms);

    match output_format {
        FileFormat::Midi => {
            #[cfg(feature = "midi")]
            {
                if verbose {
                    println!("Writing MIDI file: {}", output_file);
                }
                midi::convert_mtxt_to_midi(&mtxt_file, output_file, verbose)
                    .context("Failed to convert MTXT to MIDI")?;
            }
            #[cfg(not(feature = "midi"))]
            {
                anyhow::bail!("MIDI support is not enabled. Compile with --features midi");
            }
        }
        FileFormat::Mtxt => {
            if verbose {
                println!("Writing MTXT file: {}", output_file);
            }
            let output_content = format!("{}", mtxt_file);
            std::fs::write(output_file, output_content)
                .with_context(|| format!("Failed to write output file: {}", output_file))?;
        }
    }

    Ok(())
}

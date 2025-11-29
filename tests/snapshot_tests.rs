use mtxt::parse_mtxt;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_snapshots() {
    let mut snapshots_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    snapshots_dir.push("tests");
    snapshots_dir.push("snapshots");

    let entries = fs::read_dir(&snapshots_dir).expect("Failed to read snapshots directory");

    let mut failed = false;

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("mtxt") {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.ends_with(".in.mtxt") {
                println!("Testing snapshot: {}", file_name);
                if let Err(e) = run_snapshot_test(&path) {
                    println!("Snapshot test failed for {}: {}", file_name, e);
                    failed = true;
                }
            }
        }
    }

    if failed {
        panic!("Some snapshot tests failed");
    }
}

fn run_snapshot_test(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mtxt_file = parse_mtxt(&content)?;

    let out_mtxt_path = path.with_file_name(
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".in.mtxt", ".out.mtxt"),
    );
    let out_mtxt_content = format!("{}", mtxt_file);

    verify_or_update(&out_mtxt_path, &out_mtxt_content)?;

    let out_events_path = path.with_file_name(
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".in.mtxt", ".out.events"),
    );
    let out_events_content = mtxt_file
        .get_output_records()
        .iter()
        .map(|r| format!("{}", r))
        .collect::<Vec<String>>()
        .join("\n");

    verify_or_update(&out_events_path, &out_events_content)?;

    Ok(())
}

fn verify_or_update(
    path: &PathBuf,
    expected_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        let existing_content = fs::read_to_string(path)?;
        let normalized_existing = existing_content.replace("\r\n", "\n");
        let normalized_expected = expected_content.replace("\r\n", "\n");

        if normalized_existing != normalized_expected {
            return Err(format!(
                "Content mismatch for {:?}.\nExpected:\n---\n{}\n---\nActual:\n---\n{}\n---",
                path, normalized_expected, normalized_existing
            )
            .into());
        }
    } else {
        println!("Creating new snapshot file: {:?}", path);
        fs::write(path, expected_content)?;
    }
    Ok(())
}

use std::{fs::File, io::Write, path::PathBuf, process::Command};

use eyre::{OptionExt, Result};

fn main() -> Result<()> {
    let client_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_eyre("No parent")?
        .join("client");

    let buf = PathBuf::from(std::env::var("OUT_DIR")?).join("client.tar.gz");
    let output = Command::new("tar")
        .args([
            "czf",
            &buf.to_string_lossy(),
            "--directory",
            &client_dir.to_string_lossy(),
            ".",
        ])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

// kknd2-unpack
// Copyright (c) 2024 Matthew Costa <ucosty@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::{command, Parser, Subcommand};

use crate::decompress::decompress;
use crate::unpack::unpack;

mod unpack;
mod decompress;


#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Decompress and extract files from KKND 2 Archive
    Unpack {
        /// Archive to extract
        filename: String,

        /// Path to extract files into
        output_path: String,
    },

    /// Decompress KKND2 archive without extracting files
    Decompress {
        /// Archive to decompress
        filename: String,

        /// Output filename
        output_file: String,
    },

    /// List the files in an archive
    List {
        /// Archive filename
        filename: String,
    },
}

fn kind_to_string(kind: u32) -> Result<String, Box<dyn Error>> {
    let bytes = kind.to_le_bytes();
    Ok(String::from_utf8(bytes.to_vec())?)
}

fn unpack_command(input_file: &String, output_path: &String) -> Result<(), Box<dyn Error>> {
    let decompressed_data = decompress(input_file.as_str())?;
    let files = unpack(&decompressed_data.archive)?;

    let magic: u32 = 0xdeadc0de;

    for i in 0..files.len() {
        let data = unpack::extract_file(&decompressed_data.archive, &files[i])?;

        let base_name = Path::new(input_file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("output");

        let file_kind = kind_to_string(files[i].kind)?;
        let filename = format!("{}_{}.{}", base_name, i, file_kind);

        let output_filename = Path::new(output_path).join(filename);

        let mut output_file = File::create(output_filename)?;
        output_file.write_all(&(magic.to_le_bytes()))?;
        output_file.write_all(&files[i].offset.to_le_bytes())?;
        output_file.write_all(data.as_slice())?;
        output_file.flush()?;
    }

    Ok(())
}

fn decompress_command(input_file: &String, output_filename: &String) -> Result<(), Box<dyn Error>> {
    let decompressed_data = decompress(input_file.as_str())?;

    let mut output_file = File::create(output_filename)?;
    output_file.write_all(b"DATA")?;
    output_file.write_all(&(decompressed_data.archive.len() as u32).to_le_bytes())?;
    output_file.write_all(decompressed_data.archive.as_slice())?;
    output_file.write_all(decompressed_data.metadata.as_slice())?;
    output_file.flush()?;

    Ok(())
}

fn list_command(filename: &String) -> Result<(), Box<dyn Error>> {
    let decompressed_data = decompress(filename.as_str())?;

    let files = unpack(&decompressed_data.archive)?;

    for file in files {
        println!("{}: offset = {:#x}, size = {:#x}", kind_to_string(file.kind)?, file.offset, file.size);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Unpack { filename, output_path } => unpack_command(filename, output_path)?,
        Commands::Decompress { filename, output_file } => decompress_command(filename, output_file)?,
        Commands::List { filename } => list_command(filename)?
    }

    Ok(())
}

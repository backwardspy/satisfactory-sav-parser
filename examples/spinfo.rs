use std::{
    fs::File,
    io::{BufReader, Cursor, Read as _},
};

use binrw::BinReaderExt as _;
use flate2::read::ZlibDecoder;
use satisfactory_sav_parser::SaveFileBody;

fn main() -> anyhow::Result<()> {
    let Some(ref save_path) = std::env::args().nth(1) else {
        let cmd = std::env::args().next().unwrap();
        eprintln!("Usage: {cmd} <path>");
        std::process::exit(1);
    };

    let mut parser = {
        let Ok(file) = File::open(save_path) else {
            eprintln!("Failed to open file: {save_path}");
            std::process::exit(1);
        };
        satisfactory_sav_parser::Parser::new(BufReader::new(file))
    };

    let header = parser.read_header().expect("can read header");
    println!("header: {header:#?}");

    let mut chunks = vec![];
    while let Some(chunk) = parser.read_compressed_body_chunk()? {
        chunks.push(chunk);
    }

    println!(
        "read {} chunks totalling {} kB ({} kB uncompressed)",
        chunks.len(),
        chunks
            .iter()
            .map(|c| c.compressed_size_summary)
            .sum::<i64>()
            / 1024,
        chunks
            .iter()
            .map(|c| c.uncompressed_size_summary)
            .sum::<i64>()
            / 1024,
    );

    let mut body_data_raw = vec![];
    for chunk in &chunks {
        let reader = Cursor::new(&chunk.chunk_bytes);
        ZlibDecoder::new(reader)
            .read_to_end(&mut body_data_raw)
            .expect("can decompress body data");
    }

    assert_eq!(
        body_data_raw.len() as i64,
        chunks
            .iter()
            .map(|c| c.uncompressed_size_summary)
            .sum::<i64>()
    );
    println!(
        "decompressed {} kB of body data",
        body_data_raw.len() / 1024
    );

    std::fs::write("save_dump.bin", &body_data_raw)?;
    println!("wrote decompressed data to save_dump.bin");

    println!("attempting decode...");
    match Cursor::new(&body_data_raw).read_le::<SaveFileBody>() {
        Ok(body) => {
            dbg!(body);
        }
        Err(e) => eprintln!("failed to decode body:\n{e}"),
    }

    Ok(())
}

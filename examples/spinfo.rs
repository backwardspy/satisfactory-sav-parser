use std::{
    fs::File,
    io::{BufReader, Cursor, Read as _},
};

use flate2::read::ZlibDecoder;

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
        chunks.iter().map(|c| c.compressed_size).sum::<i64>() / 1024,
        chunks.iter().map(|c| c.uncompressed_size).sum::<i64>() / 1024,
    );

    let mut body_data = vec![];
    for chunk in &chunks {
        let mut decompressed = vec![0; chunk.uncompressed_size as usize];
        let reader = Cursor::new(&chunk.chunk_bytes[..]);
        ZlibDecoder::new(reader)
            .read_exact(&mut decompressed)
            .expect("can decompress chunk");
        body_data.extend_from_slice(&decompressed);
    }

    assert_eq!(
        body_data.len() as i64,
        chunks.iter().map(|c| c.uncompressed_size).sum::<i64>()
    );
    println!("decompressed {} kB of body data", body_data.len() / 1024);

    std::fs::write("save_dump.bin", body_data)?;
    println!("wrote decompressed data to save_dump.bin");

    Ok(())
}

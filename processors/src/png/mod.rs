use crate::FileProcessor;
use oxipng::{Deflater, Options, StripChunks, ZopfliOptions};
use std::io::{BufRead, BufReader, Read};
use std::num::NonZeroU64;
use std::path::Path;
use utils::error::{Result, SquashError};
use utils::SquashOptions;

pub struct PngFileProcessor {}

impl PngFileProcessor {
    const PNG_MAGIC_NUMBER: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

    const ALLOWED_CHUNKS: [[u8; 4]; 7] = [
        [0x49, 0x48, 0x44, 0x52],
        [0x50, 0x4c, 0x54, 0x45],
        [0x49, 0x44, 0x41, 0x54],
        [0x49, 0x45, 0x4e, 0x44],
        [0x74, 0x52, 0x4e, 0x53],
        [0x67, 0x41, 0x4d, 0x41],
        [0x73, 0x52, 0x47, 0x42],
    ];

    pub const fn new() -> Self {
        PngFileProcessor {}
    }

    fn check_header(&self, data: &mut impl Read, path: &Path) -> Result<()> {
        let mut magic_number = [0u8; 8];
        data.read_exact(&mut magic_number)
            .map_err(SquashError::failed_to_parse_png(path))?;
        if magic_number != Self::PNG_MAGIC_NUMBER {
            Err(SquashError::InvalidPngFile {
                path: path.display().to_string(),
                reason: "Magic number mismatch".to_string(),
            })
        } else {
            Ok(())
        }
    }
}

fn read_u32(buffer: &mut impl Read) -> std::result::Result<u32, std::io::Error> {
    let mut number = [0u8; 4];
    buffer.read_exact(&mut number)?;
    Ok(u32::from_be_bytes(number))
}

fn read_type(buffer: &mut impl Read) -> std::result::Result<[u8; 4], std::io::Error> {
    let mut number = [0u8; 4];
    buffer.read_exact(&mut number)?;
    Ok(number)
}

impl FileProcessor for PngFileProcessor {
    fn can_process(&self, path: &Path) -> bool {
        match path.extension() {
            None => {
                println!("{}", path.display());
                false
            }
            Some(str) => match str.to_str().unwrap_or("").to_lowercase().as_str() {
                "png" => true,
                _ => false,
            },
        }
    }

    fn process(&self, vec: Vec<u8>, path: &Path, options: &SquashOptions) -> Result<Vec<u8>> {
        if options.oxipng {
            let mut options = Options::max_compression();
            options.strip = StripChunks::Safe;
            options.deflater = Deflater::Zopfli(ZopfliOptions {
                iteration_count: NonZeroU64::new(32).unwrap(),
                ..Default::default()
            });

            let result = oxipng::optimize_from_memory(&vec[..], &options).map_err(|err| SquashError::OxipngError {
                path: path.display().to_string(),
                error: format!("{}", err),
            })?;
            println!("Compressed {} from {} -> {}", path.display(), vec.len(), result.len());

            return Ok(result);
        }

        let mut new_data = Vec::<u8>::new();
        let mut data = BufReader::new(&vec[..]);

        self.check_header(&mut data, path)?;
        new_data.extend_from_slice(&Self::PNG_MAGIC_NUMBER);

        loop {
            let length = read_u32(&mut data).map_err(SquashError::failed_to_parse_png(path))?;
            let chunk_type =
                read_type(&mut data).map_err(SquashError::failed_to_parse_png(path))?;

            if Self::ALLOWED_CHUNKS.contains(&chunk_type) {
                new_data.extend_from_slice(&u32::to_be_bytes(length));
                new_data.extend_from_slice(&chunk_type);
                let mut data_blob = vec![0u8; length as usize];
                data.read_exact(&mut data_blob)
                    .map_err(SquashError::failed_to_parse_png(path))?;

                new_data.extend_from_slice(&data_blob);
                new_data.extend_from_slice(
                    &read_type(&mut data).map_err(SquashError::failed_to_parse_png(path))?,
                );

                if chunk_type == [0x49, 0x45, 0x4e, 0x44] {
                    break;
                }

                continue;
            }

            data.consume((length + 4) as usize);
        }

        Ok(new_data)
    }
}

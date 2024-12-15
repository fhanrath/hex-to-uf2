use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use anyhow::{anyhow, Result};

const UF2_MAGIC_START0: u32 = 0x0A324655; // "UF2\n"
const UF2_MAGIC_START1: u32 = 0x9E5D5157;
const UF2_MAGIC_END: u32 = 0x0AB16F30;
const ADDRESS_MASK: u32 = 0xff;
const INVERTED_ADDRESS_MASK: u32 = !ADDRESS_MASK;

struct Block {
    pub address: u32,
    pub bytes: [u8; 256],
}

impl Block {
    pub fn new(address: u32) -> Self {
        Self {
            address,
            bytes: [0; 256],
        }
    }

    pub fn encode(&self, block_no: u32, number_of_blocks: u32, family_id: Option<u32>) -> Vec<u8> {
        let flags: u32 = match family_id {
            Some(_) => 0x2000,
            None => 0x0,
        };

        // Header: equivalent to struct.pack in Python
        let mut header = vec![];
        header.extend_from_slice(&UF2_MAGIC_START0.to_le_bytes());
        header.extend_from_slice(&UF2_MAGIC_START1.to_le_bytes());
        header.extend_from_slice(&flags.to_le_bytes());
        header.extend_from_slice(&self.address.to_le_bytes());
        header.extend_from_slice(&(256u32).to_le_bytes()); // Fixed size
        header.extend_from_slice(&block_no.to_le_bytes());
        header.extend_from_slice(&number_of_blocks.to_le_bytes());
        if family_id.is_some() {
            header.extend_from_slice(&family_id.unwrap().to_le_bytes());
        } else {
            header.extend_from_slice(&(0x00 as u32).to_le_bytes());
        }

        // Add the block's data
        header.extend_from_slice(&self.bytes);

        // Pad with 0x00 to make it 512 bytes - 4 (for the footer)
        while header.len() < 512 - 4 {
            header.push(0x00);
        }

        header.extend_from_slice(&UF2_MAGIC_END.to_le_bytes());

        header
    }
}

fn main() {
    hex_to_uf2_file(
        Path::new("./test/fenris-rmk-central.hex"),
        Path::new("./test/fenris-rmk-central.uf2"),
    )
    .unwrap();
}

fn hex_to_uf2(hex_lines: impl Iterator<Item = String>, family_id: Option<u32>) -> Result<Vec<u8>> {
    let mut upper: u32 = 0;
    let mut app_start_address: Option<u32> = None;

    let mut current_block: Option<Block> = None;

    let mut blocks: Vec<Block> = vec![];

    for hex_line in hex_lines {
        if !hex_line.starts_with(':') {
            continue;
        }

        let mut binary_line = vec![];
        let mut i = 1;

        while i < hex_line.len() - 1 {
            let byte = u8::from_str_radix(&hex_line[i..i + 2], 16).unwrap_or(0);
            binary_line.push(byte);
            i += 2;
        }

        let byte_count = binary_line[0] as usize;
        if binary_line.len() - 5 != byte_count {
            return Err(anyhow!("Invalid HEX line: mismatched byte count"));
        }

        if binary_line.len() < 4 {
            return Err(anyhow!(
                "Line with less than 4 encoded hex chars + 1 byte leads to undefined behaviour!"
            ));
        }

        match binary_line[3] {
            0 => {
                let mut address =
                    upper + (((binary_line[1] as u32) << 8) | (binary_line[2] as u32));
                if app_start_address.is_none() {
                    app_start_address = Some(address);
                }
                // skip first 4 and last item
                for index in 4..binary_line.len() - 1 {
                    let byte = &binary_line[index];
                    if current_block.is_none()
                        || (current_block.as_ref().unwrap().address & INVERTED_ADDRESS_MASK)
                            != (address & INVERTED_ADDRESS_MASK)
                    {
                        if let Some(block) = current_block.take() {
                            blocks.push(block);
                        }
                        current_block = Some(Block::new(address & INVERTED_ADDRESS_MASK));
                    }
                    let current_mut_block = current_block
                        .as_mut()
                        .expect("current_block should always be some here");

                    current_mut_block.bytes[(address & ADDRESS_MASK) as usize] = *byte;
                    address += 1;
                }
            }
            1 => break, // End of file
            2 => {
                upper = (((binary_line[4] as u32) << 8) | (binary_line[5] as u32)) << 4;
            }
            4 => {
                upper = (((binary_line[4] as u32) << 8) | (binary_line[5] as u32)) << 16;
            }
            _ => {
                // do nothing
            }
        }
    }

    if let Some(block) = current_block.take() {
        // Add last block
        blocks.push(block);
    }

    let number_of_blocks = blocks.len() as u32;

    Ok(blocks
        .iter()
        .enumerate()
        .flat_map(|(i, block)| {
            block
                .encode(i as u32, number_of_blocks, family_id)
                .into_iter()
        })
        .collect())
}

fn hex_to_uf2_file(hex_file: &Path, output_path: &Path) -> Result<()> {
    let binary_buffer = BufReader::new(File::open(hex_file).expect("Couldn't open input file!"));
    let family = None;

    let uf2_buffer = hex_to_uf2(
        binary_buffer
            .lines()
            .map(|line| line.expect("error reading line")),
        family,
    )
    .expect("Error converting hex to uf2");

    let mut file =
        File::create(output_path).expect("Error creating or overwriting destination file");

    file.write_all(&uf2_buffer)
        .expect("Error writing to destination");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn compare_to_python() {
        hex_to_uf2_file(
            Path::new("./test/fenris-rmk-central.hex"),
            Path::new("./test/fenris-rmk-central.uf2"),
        )
        .unwrap();

        let rust_reader = BufReader::new(File::open("./test/fenris-rmk-central.uf2").unwrap());
        let python_reader = BufReader::new(File::open("./test/fenris-rmk-central_py.uf2").unwrap());

        let mut rust_bytes = rust_reader.bytes();
        let mut python_bytes = python_reader.bytes();

        let mut position: usize = 0;

        loop {
            match (rust_bytes.next(), python_bytes.next()) {
                (None, None) => break,
                (Some(_), None) => {
                    assert!(false, "rust_bytes is longer than python_bytes");
                    break;
                }
                (None, Some(_)) => {
                    assert!(false, "python_bytes is longer than rust_bytes");
                    break;
                }
                (Some(rust_byte), Some(python_byte)) => {
                    assert_eq!(
                        rust_byte.unwrap(),
                        python_byte.unwrap(),
                        "Difference at {position}, hex: {position:X}"
                    );
                }
            }

            position += 1;
        }
    }
}

use std::io::{self, Read, Write};
use byteorder::{ReadBytesExt, LittleEndian};

mod brr;
fn main() {
    let mut buf: [u8; 512] = [0x00; 512];
    let mut block_buf: [i16; 16] = [0; 16];
    let mut encoder = brr::Encoder::new();
    let mut stdin = io::stdin();
    let mut total_bytes = 0;
    let mut first_block = true;
    let mut mbytes = 0;

    loop {
        /* Add some data into the input buffer */        
        let bytes_read = stdin.read(&mut buf).unwrap();
        
        if bytes_read == 0 {
            break;
        }

        total_bytes += bytes_read;

        /* If there's more than 32 bytes in the buffer, grab a block and encode it */
        for mut block in buf[..bytes_read].chunks(32) {
            if block.len() == 32 {            
                let _ = block.read_i16_into::<LittleEndian>(&mut block_buf);
                let mut pcm_data: [i32; 16] = [0; 16];                

                for i in 0..16 {
                    pcm_data[i] = block_buf[i] as i32;
                }
                
                if first_block {
                    let mut initial_block = false;
                    for p in &pcm_data {
                        initial_block |= *p != 0;
                    }

                    if initial_block {
                        let _ = std::io::stdout().write(&[0u8; 9]);
                    }
                    
                    first_block = false;
                }
                let brr_data = encoder.adpcm_block_mash(&pcm_data, false, false);
                let _ = std::io::stdout().write(brr_data);
            }
        }

        if total_bytes / 1024 / 1024 > mbytes {
            mbytes = total_bytes / 1024 / 1024;
            eprint!("\x1B[1A\x1B[255D\x1B[2KEncoding - Read {:?} MB, Written {:?} MB\n", mbytes, mbytes as f64 * (9.0/32.0));
        }
    }    
}

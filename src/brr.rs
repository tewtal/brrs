/*

This code is based on BRRtools (https://github.com/Optiroc/BRRtools)

by Bregalad. Special thanks to Kode54.  
Bug fixes and improvements by jimbo1qaz and Optiroc.  

BRRtools are currently the most evolved tools to convert between standard RIFF .wav format and SNES's built-in BRR sound format.
They have many features never seen before in any other converter, and are open source.

Versions up to 2.1 used to be coded in Java, requiring a Java virtual machine to run.
Because this was an useless layer of abstraction which is only useful when developing, the program was rewritten to not need Java any longer.

I heavily borrowed encoding algorithms from Kode54, which himself heavily borrowed code from some other ADPCM encoder.
This is freeware, feel free to redistribute/improve but DON'T CLAIM IT IS YOUR OWN WORK THANK YOU.

*/

pub struct Encoder
{
    brr: [u8; 9],
    wrap_en: bool,
    p1: i16,
    p2: i16,
    loop_filter: i32,
    loop_p1: i16,
    loop_p2: i16,
}

impl Encoder
{
    pub fn new() -> Encoder {
        Encoder {
            brr: [0; 9],
            wrap_en: true,
            p1: 0,
            p2: 0,
            loop_filter: 0,
            loop_p1: 0,
            loop_p2: 0,
        }
    }

    fn clamp_16(&mut self, val: i32) -> i16 {
        if ((val as i16) as i32) != val { (0x7fff - (val >> 24)) as i16 } else { val as i16 }
    }

    fn get_brr_prediction(&mut self, filter: u8, p1: i16, p2: i16) -> i32 {
        let mut p: i32;
        let p1 = p1 as i32;
        let p2 = p2 as i32;

        match filter {
            0 => 0,
            1 => {
                p = p1;
                p -= p1 >> 4;                
                p
            }
            2 => {
                p = p1 << 1;
                p += (-(p1 + (p1 << 1))) >> 5;
                p -= p2;
                p += p2 >> 4;
                p
            }
            3 => {
                p = p1 << 1;
                p += (-(p1 + (p1 << 2) + (p1 << 3))) >> 6;
                p -= p2;
                p += (p2 + (p2 << 1)) >> 4;
                p
            }
            _ => 0
        }        
    }

    fn adpcm_mash(&mut self, shiftamount: u32, filter: u8, pcm_data: &[i32; 16], write: bool, endpoint: bool) -> f64 {
        let mut d2: f64 = 0.0;
        
        let mut l1 = self.p1 as i32;
        let mut l2 = self.p2 as i32;
        let step = 1 << shiftamount;

        let mut vlin: i32;
        let mut d: i32;
        let mut da: i32;
        let mut dp: i32;
        let mut c: i32;

        for i in 0..16 {
            vlin = self.get_brr_prediction(filter, l1 as i16, l2 as i16) >> 1;
            d = (pcm_data[i] >> 1) - vlin;
            da = d.abs();
            
            if self.wrap_en && da > 16384 && da < 32768 {
                d = d - (32768 * (d >> 24));
                //if write { eprint!("Wrapping was used!, {:?}", d) };
            }

            dp = d + (step << 2) + (step >> 2);
            c = 0;

            if dp > 0 {
                if step > 1 {
                    c = dp / (step / 2);
                } else {
                    c = dp * 2;
                }
                if c > 15 {
                    c = 15;
                }
            }

            c -= 8;
            dp = (c << (shiftamount as i32)) >> 1;

            if shiftamount > 12 {
                dp = (dp >> 14) & !0x7ff;
            }

            c &= 0x0f;

            l2 = l1;
            l1 = self.clamp_16(vlin + dp) as i32 * 2;

            d = pcm_data[i] - l1;
            d2 += d as f64 * d as f64;

            if write {
                self.brr[1 + (i >> 1)] |= if (i & 1) == 1 { c } else { c << 4 } as u8;
            }
        }

        if endpoint {
            match self.loop_filter {
                0 => d2 /= 16.0,
                1 => {
                    d = l1 - self.loop_p1 as i32;
                    d2 += d as f64 * d as f64;
                    d2 /= 17.0;
                },
                _ => {
                    d = l1 - self.loop_p1 as i32;
                    d2 += d as f64 * d as f64;
                    d = l2 - self.loop_p2 as i32;
                    d2 += d as f64 * d as f64;
                    d2 /= 18.0;
                }
            };
        } else {
            d2 /= 16.0;
        }

        if write {
            self.p1 = l1 as i16;
            self.p2 = l2 as i16;
            self.brr[0] = ((shiftamount as u8) << 4) | (filter << 2);
            
            if endpoint {
                self.brr[0] |= 1;
            }
        }

        d2
    }

    pub fn adpcm_block_mash(&mut self, pcm_data: &[i32; 16], looppoint: bool, endpoint: bool) -> &[u8; 9] {
        let mut smin = 0;
        let mut kmin = 0;
        let mut dmin = std::f64::INFINITY;

        self.brr = [0; 9];

        for s in 0..13 {
            for k in 0..4 {
                let d = self.adpcm_mash(s, k, pcm_data, false, endpoint);
                if d < dmin {
                    kmin = k;
                    dmin = d;
                    smin = s;
                }
            }
        }

        if looppoint {
            self.loop_filter = kmin as i32;
            self.loop_p1 = self.p1;
            self.loop_p2 = self.p2;
        }

        self.adpcm_mash(smin, kmin, pcm_data, true, endpoint);
        &self.brr
    }
}
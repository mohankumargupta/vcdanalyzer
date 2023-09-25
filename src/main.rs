use std::time::{Duration, Instant};
use std::{error::Error, fs::File};
mod generator;
mod vcdwrapper;
use std::io::Read;
use vcdwrapper::VCDWrapper;

// Only support vcd generated from Wokwi itself using the logic analyzer
// TODO: Support VCD generated from sigrok/pulseview
// https://github.com/yne/vcd/blob/master/samples/libsigrok.vcd

#[derive(Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
struct Signal {
    timestamp: u64,
    port: Vec<String>,
    values: Vec<u8>,
}

impl Signal {
    fn new(t: u64) -> Self {
        Self {
            timestamp: t,
            port: Vec::new(),
            values: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    //vcd_to_wokwi_chip()
    let start = Instant::now();
    let input_file = "waveform.vcd";
    let mut f = File::open(input_file).expect("Unable to open the file");
    let mut contents = String::new();

    f.read_to_string(&mut contents)
        .expect("Unable to read the file");
    let v = VCDWrapper::from_string(contents.as_bytes());
    if let Some(generated_code) = v.vcd_to_wokwi_chip()? {
        println!("{}", generated_code);
    }

    let duration: Duration = start.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);
    Ok(())
}

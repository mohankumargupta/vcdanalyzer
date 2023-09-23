use genco::{fmt, prelude::*};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::File,
    io::BufReader,
};
use vcd::{Parser, Scope, ScopeItem};

const ROOT_SCOPE: &str = "logic";

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
    let mut reader = Parser::new(BufReader::new(File::open("waveform.vcd")?));
    let header = reader.parse_header()?;
    let scope: Option<&Scope> = header.find_scope(&[ROOT_SCOPE]);

    let items: Vec<&ScopeItem> = if let Some(scope) = scope {
        scope.items.iter().collect()
    } else {
        Vec::new()
    };

    let mut ports = HashMap::new();
    let mut signal = BTreeMap::new();

    for &c in &items {
        if let ScopeItem::Var(i) = c {
            ports.insert(i.code.to_string(), i.reference.to_string());
        }
    }

    let mut timestamp: u64 = 0;

    for command_result in reader {
        let command = command_result?;
        use vcd::Command::*;

        match command {
            Timestamp(t) => {
                //println!("Time is {t}");
                timestamp = t;
            }
            ChangeScalar(i, v) => {
                let boo = signal
                    .entry(timestamp)
                    .or_insert_with(|| Signal::new(timestamp));
                boo.port
                    .push(ports.get(&i.to_string()).unwrap().to_string());
                boo.values.push(v.to_string().parse::<u8>().unwrap());

                //println!("{:?}", signal);
            }
            _ => (),
        }
    }
    //println!("{:?}", signal);
    println!("{:?}", signal.values());

    let malloc = &c::include_system("stdlib.h", "malloc");
    let printf = &c::include_system("stdio.h", "printf");

    let day = "tuesday";
    let name = "George";

    let tokens = quote! {
        typedef struct {
            // TODO: Put your chip variables here
        } chip_state_t;

        const char* greet_user() {
            return $(quoted(format!("Hello {}!", name)));
        }

        int main() {
            const char* current_day = $(quoted(day));
            chip_state_t *chip = $malloc(sizeof(chip_state_t));
            $printf("Hello from custom chip!\n");
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<C>();
    let config = c::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;

    Ok(())
}

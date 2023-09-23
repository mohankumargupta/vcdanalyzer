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
    println!("{:?}", ports);
    let ports_values = ports.values();
    println!("{:?}", ports_values);

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
    //let pin_init = &c::include("wokwi-api.h", "pin_init");
    //let tristate = "tristatelevel {}=current_pulse.{};\n";

    let tokens = quote! {
        #include "wokwi-api.h"

        typedef struct {
            int index;
            timer_t timer;
        $(for n in ports_values.clone() => $(format!("pin_t {};\n", n)))
        } chip_state_t;

        typedef struct {
        unsigned long timestamp;
        $(for n in ports_values.clone() => bool $(format!("{};\n", n)))
        } pulse;

        typedef enum {
            dontcarelevel=-1,
            lowlevel=0,
            highlevel=1,
        } tristatelevel;

        const pulse pulse_train[] = {
            {.timestamp =0, .D0 = lowlevel, .D1 = lowlevel, .D2 = lowlevel, .D3 = lowlevel },
            {.timestamp = 15000, .D0 = lowlevel, .D1 = lowlevel, .D2 = lowlevel, .D3 = lowlevel },
            {.timestamp = 30000, .D0 = highlevel },
        };

        const unsigned int  NUMBER_OF_PULSES = sizeof(pulse_train)/sizeof(pulse);

        //const char* greet_user() {
        //    return $quoted($(format!("Hello {}!", name)));
        //}

        void chip_timer_event(void *user_data) {
            chip_state_t *chip = (chip_state_t *)user_data;
            pulse current_pulse = pulse_train[chip->index];

            unsigned long t = current_pulse.timestamp;
            $(for n in ports_values.clone() => $(
                format!("tristatelevel {}=current_pulse.{};\n", n, n)

            )
            )

            $(for n in ports_values.clone() => $(
                format!("if ({} != dontcarelevel ) {}pin_write(chip->{}, {});{}\n", n,"{",n, n,"}")
            ))




            //$printf("chip_timer_event! timestamp:%d\n", NUMBER_OF_PULSES);
            unsigned long sim_time = (unsigned long) get_sim_nanos();
            $printf("sim time:%lu\n", sim_time);
            $printf("index: %d\n", chip->index);
            $printf("current timestamp: %lu\n", t);
            chip->index = chip->index + 1;
            if ((chip->index) != NUMBER_OF_PULSES) {
                unsigned long next_pulse = pulse_train[chip->index].timestamp - t;
                $printf("next timestamp: %lu\n", pulse_train[chip->index].timestamp);
                timer_start_ns(chip->timer, next_pulse, false);

            }
        }

        void chip_init() {

            //const char* current_day = $(quoted(day));
            chip_state_t *chip = $malloc(sizeof(chip_state_t));

            $(for n in ports_values.clone() => $(format!("chip->{} = pin_init(\"{}\", OUTPUT);\n", n, n)))

            timer_config_t timer_config = {
                .callback = chip_timer_event,
                .user_data = chip
            };

            chip->timer = timer_init(&timer_config);
            timer_start_ns(chip->timer, 0, false);

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

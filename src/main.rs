use genco::{fmt, prelude::*};
use std::{collections::BTreeMap, error::Error, fs::File, io::BufReader};
use vcd::{Header, Parser, Scope, ScopeItem};

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

/*
fn found_root_scope() {
    let scope: Option<&Scope> = header.find_scope(&[root_scope]);

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
    let timer_init = &c::include("wokwi-api.h", "timer_init");
    let timer_start_ns = &c::include("wokwi-api.h", "timer_start_ns");

    let tokens = quote! {
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
                $timer_start_ns(chip->timer, next_pulse, false);

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

            chip->timer = $timer_init(&timer_config);
            $timer_start_ns(chip->timer, 0, false);

            $printf("Hello from custom chip!\n");
        }
    };

    //let stdout = std::io::stdout();
    //let mut w = fmt::IoWriter::new(stdout.lock());

    //could also use std::fs::File

    let mut w = fmt::IoWriter::new(Vec::<u8>::new());

    let fmt = fmt::Config::from_lang::<C>();
    let config = c::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    let vector = w.into_inner();
    let string = std::str::from_utf8(&vector)?;
    println!("{}", string);
}
*/
fn find_root_scope_name(header: &Header) -> Option<&String> {
    header.items.iter().find_map(|item| {
        if let ScopeItem::Scope(scope) = item {
            Some(&scope.identifier)
        } else {
            None
        }
    })
}

fn find_ports(header: &Header, root_scope: &str) -> BTreeMap<String, String> {
    let scope: Option<&Scope> = header.find_scope(&[root_scope]);

    let items: Vec<&ScopeItem> = if let Some(scope) = scope {
        scope.items.iter().collect()
    } else {
        Vec::new()
    };

    let mut ports = BTreeMap::new();

    for &c in &items {
        if let ScopeItem::Var(i) = c {
            ports.insert(i.code.to_string(), i.reference.to_string());
        }
    }

    //println!("{:?}", ports);
    ports
}

fn capture_signals(
    reader: Parser<BufReader<File>>,
    ports: &BTreeMap<String, String>,
) -> Result<BTreeMap<u64, Signal>, Box<dyn Error>> {
    let mut timestamp: u64 = 0;
    let mut signal = BTreeMap::new();

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
            _ => {}
        }
    }
    //println!("{:?}", signal);
    Ok(signal)
}

fn generate_wokwi_chip(
    ports: &BTreeMap<String, String>,
    _signals: &BTreeMap<u64, Signal>,
) -> Result<String, Box<dyn Error>> {
    let ports_values = ports.values();

    let malloc = &c::include_system("stdlib.h", "malloc");
    let printf = &c::include_system("stdio.h", "printf");
    //let pin_init = &c::include("wokwi-api.h", "pin_init");
    let timer_init = &c::include("wokwi-api.h", "timer_init");
    let timer_start_ns = &c::include("wokwi-api.h", "timer_start_ns");

    let tokens = quote! {
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
                $timer_start_ns(chip->timer, next_pulse, false);

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

            chip->timer = $timer_init(&timer_config);
            $timer_start_ns(chip->timer, 0, false);

            $printf("Hello from custom chip!\n");
        }
    };

    //let stdout = std::io::stdout();
    //let mut w = fmt::IoWriter::new(stdout.lock());

    //could also use std::fs::File

    let mut w = fmt::IoWriter::new(Vec::<u8>::new());

    let fmt = fmt::Config::from_lang::<C>();
    let config = c::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    let vector = w.into_inner();
    let string = std::str::from_utf8(&vector)?;

    Ok(string.to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = Parser::new(BufReader::new(File::open("waveform.vcd")?));
    let header = reader.parse_header()?;
    if let Some(root_scope) = find_root_scope_name(&header) {
        let ports = find_ports(&header, &root_scope);
        println!("{:?}", &ports);
        let signals = capture_signals(reader.into_iter(), &ports)?;
        println!("{:?}", &signals);
        let wokwi_chip_code = generate_wokwi_chip(&ports, &signals)?;
        println!("{}", wokwi_chip_code);
    } else {
        // Handle the case where the root scope is not found
    }

    /*
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
        let timer_init = &c::include("wokwi-api.h", "timer_init");
        let timer_start_ns = &c::include("wokwi-api.h", "timer_start_ns");

        let tokens = quote! {
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
                    $timer_start_ns(chip->timer, next_pulse, false);

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

                chip->timer = $timer_init(&timer_config);
                $timer_start_ns(chip->timer, 0, false);

                $printf("Hello from custom chip!\n");
            }
        };

        //let stdout = std::io::stdout();
        //let mut w = fmt::IoWriter::new(stdout.lock());

        //could also use std::fs::File

        let mut w = fmt::IoWriter::new(Vec::<u8>::new());

        let fmt = fmt::Config::from_lang::<C>();
        let config = c::Config::default();

        tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
        let vector = w.into_inner();
        let string = std::str::from_utf8(&vector)?;
        println!("{}", string);
    */
    Ok(())
}

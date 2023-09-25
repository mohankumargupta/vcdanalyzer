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

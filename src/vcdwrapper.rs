use std::io::BufReader;
use std::{collections::BTreeMap, error::Error};
use vcd::Command::{self, *};
use vcd::{Header, Parser, Scope, ScopeItem};

use crate::Signal;

use crate::generate_wokwi_chip;

pub struct VCDWrapper<'a> {
    //contents: Parser<BufReader<&'a [u8]>>,
    contents: &'a [u8],
}

/*
impl<T: AsRef<Path>, U> From<T> for VCDWrapper<U> {
    fn from(value: T) -> VCDWrapper<U> {
        Self {
            reader: Parser::new(BufReader::new(File::open(value).unwrap())),
        }
    }
}
*/

impl<'a> VCDWrapper<'a> {
    pub fn from_string(contents: &'a [u8]) -> Self {
        Self { contents }
    }

    /*
    fn parse_header(&mut self) -> Result<Header, Box<dyn Error>> {
        let header = self.reader.parse_header()?;
        Ok(header)
    }
    */

    fn find_root_scope_name(&self, header: &Header) -> Option<String> {
        header.items.iter().find_map(|item| {
            if let ScopeItem::Scope(scope) = item {
                Some(scope.identifier.to_string())
            } else {
                None
            }
        })
    }

    fn find_ports(&self, header: &Header, root_scope: &str) -> BTreeMap<String, String> {
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

        ports
    }

    fn capture_signals(
        &self,
        timestamp: u64,
        command_result: Result<Command, std::io::Error>,
        signal: &mut BTreeMap<u64, Signal>,
        ports: &BTreeMap<String, String>,
    ) -> Result<Option<u64>, Box<dyn Error>> {
        let command = command_result?;

        match command {
            Timestamp(t) => Ok(Some((t))),
            ChangeScalar(i, v) => {
                let modified_signal = signal
                    .entry(timestamp)
                    .or_insert_with(|| Signal::new(timestamp));
                modified_signal
                    .port
                    .push(ports.get(&i.to_string()).unwrap().to_string());
                modified_signal
                    .values
                    .push(v.to_string().parse::<u8>().unwrap());
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn vcd_to_wokwi_chip(&self) -> Result<(), Box<dyn Error>> {
        let mut reader = Parser::new(BufReader::new(self.contents));
        let header = reader.parse_header()?;

        if let Some(root_scope) = self.find_root_scope_name(&header) {
            let ports = self.find_ports(&header, &root_scope);
            println!("{:?}", &ports);
            let mut timestamp: u64 = 0;
            let mut signal = BTreeMap::new();
            for command_result in reader {
                let result =
                    self.capture_signals(timestamp, command_result, &mut signal, &ports)?;
                if let Some(t) = result {
                    timestamp = t;
                }
            }
            println!("{:?}", &signal);

            let wokwi_chip_code = generate_wokwi_chip(&ports, &signal)?;
            println!("{}", wokwi_chip_code);
        } else {
            // Handle the case where the root scope is not found
        }
        Ok(())
    }
}

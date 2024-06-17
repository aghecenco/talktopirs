use std::{fmt::Display, str::FromStr};

use inquire::Select;


#[derive(Clone, Copy, Debug, PartialEq)]
enum Peripheral {
    Serial,
}

#[derive(Debug)]
enum PiSaysError {
    UnknownPeripheral,
}

impl FromStr for Peripheral {
    type Err = PiSaysError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().eq_ignore_ascii_case("Serial") {
            return Ok(Self::Serial);
        }
        Err(PiSaysError::UnknownPeripheral)
    }
}

impl Display for Peripheral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial => write!(f, "Serial"),
        }
    }
}

fn run_serial() {
    todo!("wait for it")
}

fn main() {
    let popts = [Peripheral::Serial];
    let popts_str = popts.iter().map(|p| format!("{}", p)).collect();
    match Select::new("What's your favorite fruit?", popts_str).prompt() {
        Ok(per_s) => {
            let per = Peripheral::from_str(&per_s).expect("Unrecognized peripheral");
            match per {
                Peripheral::Serial => run_serial(),
            }
        },
        Err(e) => panic!("Unrecognized peripheral: {}", e)
    }

}

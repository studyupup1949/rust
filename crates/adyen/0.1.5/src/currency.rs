use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Currency {
    NOK,
    SEK,
    DKK,
    ISK,
    GBP,
    EUR,
}

impl TryFrom<&str> for Currency {
    type Error = ();

    fn try_from(g: &str) -> Result<Self, Self::Error> {
        match g {
            x if x == "NOK" => Ok(Currency::NOK),
            x if x == "SEK" => Ok(Currency::SEK),
            x if x == "DKK" => Ok(Currency::DKK),
            x if x == "ISK" => Ok(Currency::ISK),
            x if x == "GBP" => Ok(Currency::GBP),
            x if x == "EUR" => Ok(Currency::EUR),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Currency::NOK => "NOK",
            Currency::SEK => "SEK",
            Currency::DKK => "DKK",
            Currency::ISK => "ISK",
            Currency::GBP => "GBP",
            Currency::EUR => "EUR",
        };
        write!(f, "{}", text)
    }
}

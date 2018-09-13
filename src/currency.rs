//!#currency
//!
//!'currency' contains a enumerator what represent the actual useful currencies for belarusian
//!national economy and also some methods for working with this currencies.
//!
//!Supporting US Dollar (USD), Euro (EUR), Russian Rouble(RUB), Belarusian Rouble (BYN)

///Represent available currencies
#[derive(PartialEq, Debug)]
pub enum Currency {
    USD,
    EUR,
    RUB,
    BYN,
}

impl Currency {
    ///Take a string what represent the currency and return exemplar of enum Currency.
    ///
    ///Read the value only in uppercase format like ``"USD"``
    ///
    ///# Errors
    ///
    ///Return err, if passed value in the lowercase or not represented by enum `Currency`
    ///
    pub fn from_str(name: &str) -> Result<Currency, String> {
        match name {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "RUB" => Ok(Currency::RUB),
            "BYN" => Ok(Currency::BYN),
            s => Err(format!("mismatching currency {}, check definition of enum Currency", s))
        }
    }
    ///Returning value of internal ID of currency used in API of
    ///National Bank of Republic of Belarus site
    ///
    ///# Warning
    ///
    ///Return 0 for `Currency::BYN`, what is not the official API of NBRB site
    ///
    pub fn get_id(&self) -> u16 {
        match self {
            Currency::USD => 145,
            Currency::EUR => 19,
            Currency::RUB => 298,
            Currency::BYN => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_with_usd() {
        let usd = Currency::from_str("USD").unwrap();
        assert_eq!(usd, Currency::USD);
    }
}

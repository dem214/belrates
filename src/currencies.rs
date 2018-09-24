//! Contains a enumerator with avaiable currencies.

/// Available currencies represents the actual useful currencies for belarusian
/// national economy and also some methods for working with this currencies.
/// Contains code of currencies by
/// [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217).
#[derive(PartialEq, Debug)]
pub enum Currency {
    /// United States dollar
    USD,
    /// Euro
    EUR,
    /// Russian ruble
    RUB,
    /// Belarusian ruble
    BYN,
    /// British pound sterling
    GBT,
    /// Ukrainian hryvnia
    UAH,
    /// Polish zloty
    PLN,
    /// Chinese yuan
    CNY,
    /// Japanese yen
    JPY,
    /// Kazakhstani tenge
    KZT,
    /// Swiss franc
    CHF,
    /// Canadian dollar
    CAD,
}

impl Currency {

    /// Take a string what represent the currency and return exemplar of enum Currency.
    ///
    /// Read the value only in uppercase format by
    /// [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217)
    /// like ``"USD"``.
    ///
    /// # Errors
    ///
    /// Return `Err`, if passed value in the lowercase or not represented by enum `Currency`.
    ///
    /// # Examples
    /// ```
    /// # use belrates::*;
    /// let usd = Currency::from_str("USD").unwrap();
    ///
    /// assert_eq!(usd, Currency::USD);
    /// ```
    pub fn from_str(name: &str) -> Result<Currency, String> {
        match name {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "RUB" => Ok(Currency::RUB),
            "BYN" => Ok(Currency::BYN),
            "GBT" => Ok(Currency::GBT),
            "UAH" => Ok(Currency::UAH),
            "PLN" => Ok(Currency::PLN),
            "CNY" => Ok(Currency::CNY),
            "JPY" => Ok(Currency::JPY),
            "KZT" => Ok(Currency::KZT),
            "CHF" => Ok(Currency::CHF),
            "CAD" => Ok(Currency::CAD),
            s => Err(format!("mismatching currency {}, check definition of enum Currency", s)),
        }
    }

    /// Returning value of internal ID of currency used in API of
    /// National Bank of Republic of Belarus site.
    ///
    /// # Warning
    ///
    /// Return `0` for `Currency::BYN`, what is not the official API of NBRB site.
    ///
    /// # Examples
    ///
    /// ```
    /// # use belrates::*;
    /// let usd_id: u16 = Currency::USD.get_id();
    ///
    /// assert_eq!(usd_id, 145);
    /// ```
    pub fn get_id(&self) -> u16 {
        match self {
            Currency::USD => 145,
            Currency::EUR => 19,
            Currency::RUB => 298,
            Currency::GBT => 143,
            Currency::UAH => 290,
            Currency::PLN => 293,
            Currency::CNY => 304,
            Currency::JPY => 295,
            Currency::KZT => 301,
            Currency::CAD => 23,
            Currency::CHF => 130,
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

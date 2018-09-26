//! Belrates contains useful functions helps you to get actual exhange rate of foreign currencies
//! in Belarusian rouble - national currency of Republic of Belarus.
//!
//! Exchange rates provided by API of site of National Bank of Republic of Belarus
//! [nbrb.by](http://www.nbrb.by).
//!

extern crate reqwest;

use std::u32;
use reqwest::StatusCode;

pub mod currencies;

#[doc(inline)]
pub use currencies::Currency;

// URL of `nbrb.com` server API
const URL_RATES: &'static str = "http://www.nbrb.by/API/ExRates/Rates/";

/// Represent server response in useful form.
#[derive(Debug, PartialEq)]
pub struct Rate {
    pub id: u32,
    pub date: String,
    pub abb: Currency,
    pub scale: u32,
    pub name: String,
    pub rate: f32,
}

impl Rate {

    /// Parsing server's JSON response into `Rate` struct instance.
    ///
    /// Example of parsed `s: String` param:
    ///
    /// `{"Cur_ID":145,"Date":"2018-09-21T00:00:00","Cur_Abbreviation":"USD","Cur_Scale":1,
    /// "Cur_Name":"Доллар США","Cur_OfficialRate":2.0884}`
    ///
    /// # Errors
    /// Return `Err(&'static str)` with message in `&str` which disribe type of `Err`
    /// if failing to parse fields of input param.
    ///
    /// # Examples
    /// ```
    /// # use belrates::*;
    /// let resp = "{\"Cur_ID\":145,\"Date\":\"2018-09-21T00:00:00\",\"Cur_Abbreviation\":\"USD\",
    /// \"Cur_Scale\":1,\"Cur_Name\":\"Доллар США\",\"Cur_OfficialRate\":2.0884}".to_string();
    /// let resp_rate = Rate::from_string(resp).unwrap();
    /// let ex_rate = Rate {
    ///     id: 145,
    ///     date: "2018-09-21T00:00:00".to_string(),
    ///     abb: Currency::USD,
    ///     scale: 1,
    ///     name: "Доллар США".to_string(),
    ///     rate: 2.0884,
    /// };
    /// assert_eq!(resp_rate, ex_rate);
    /// ```
    pub fn from_string(s: String) -> Result<Rate, String> {
        let s = s.trim_left_matches('{').trim_right_matches('}');
        let table: Vec<&str> = s.split(',').collect();

        let row: Vec<&str> = table[0].splitn(2, ':').collect();
        let id = row[1].parse::<u32>();
        let id = match id {
            Ok(u) => u,
            Err(_) => return Err("Error while parsing Cur_ID into u32".to_string()),
        };

        let row: Vec<&str> = table[1].splitn(2, ':').collect();
        let date = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[2].splitn(2, ':').collect();
        let abb = Currency::from_str(&row[1].trim_matches('"').to_string());
        let abb = match abb {
            Ok(u) => u,
            Err(e) => return Err(format!("Error while parsing Cur_Abbreviation into u32: {}", e)),
        };

        let row: Vec<&str> = table[3].splitn(2, ':').collect();
        let scale= row[1].parse::<u32>();
        let scale = match scale {
            Ok(u) => u,
            Err(_) => return Err("Error while parsing Cur_Scale into u32".to_string()),
        };

        let row: Vec<&str> = table[4].splitn(2, ':').collect();
        let name = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[5].splitn(2, ':').collect();
        let rate = row[1].parse::<f32>();
        let rate = match rate {
            Ok(u) => u,
            Err(_) => return Err("Error while parsing Cur_Rate into f32".to_string()),
        };;

        Ok(Rate{id, date, abb, scale, name, rate})
    }
    /// Make request to the server, parse then return in `Rate` struct contains
    /// today exchange rate.
    ///
    /// # Examples
    /// ```
    /// # use belrates::*;
    /// let rate = Rate::from_server_today(&Currency::USD).unwrap();
    /// ```
    pub fn from_server_today(cur: &Currency) -> Result<Rate, String> {
        Rate::from_string(get_from_server(cur)?)
    }

    /// Make request to the server, parse then return result in `Rate` struct
    /// contains exchange rate on date.
    ///
    /// # Warning
    ///
    ///Need `date` in ISO 8601 format like `"YYYY-MM-DD"`.
    ///
    /// # Examples
    /// ```
    /// # use belrates::*;
    /// let rate = Rate::from_server_date(&Currency::USD, "2018-04-20").unwrap();
    /// ```
    pub fn from_server_date(cur: &Currency, date: &str) -> Result<Rate, String>
    {
        match date_get(cur, date) {
            Ok(val) => Rate::from_string(val),
            Err(e) => Err(format!("cannot get rate on the date: {}", e)),
        }
    }

    /// Return actual rate that represent official rate devided by official currency scale of NBRB.
    ///
    /// `self.act_rate == self.rate / self.scale`
    ///
    /// That means a cost of 1 unit of the foreing currency in the belarusian rubles.
    ///
    /// # Examples
    ///
    /// ```
    /// # use belrates::*;
    /// use std::f32;
    /// let some_rate = Rate {
    ///    # id: 1,
    ///    # date: "today".to_string(),
    ///    # abb: Currency::RUB,
    ///    # name: "Рубль".to_string(),
    ///    // snip..
    ///    scale: 100,
    ///    rate: 3.14,
    /// };
    /// let dif = (some_rate.act_rate() - 0.0314_f32).abs();
    ///
    /// assert!(dif <= f32::EPSILON);
    /// ```
    pub fn act_rate(&self) -> f32 {
        self.rate / self.scale as f32
    }
}

/// Returns response from API server in raw JSON string format.
/// Response contains exchange rate of `cur: Currency` today.
///
/// # Errors
/// Return `Err<String>` if you put `Currency::BYN` in `cur` param.
///
/// # Examples
/// ```
/// # use belrates::*;
/// let json = get_from_server(&Currency::USD).unwrap();
/// ```
fn get_from_server(cur: &Currency) -> Result<String, String> {
    //we can't get the rate of BYN in BYN, its a silly
    if *cur == Currency::BYN {
        return Err("cannot get currency for BYN".to_string());
    }
    let url = format!("{}{}",URL_RATES, cur.get_id());
    //send request to server
    request(&url)
}
/// Returns response from API server in raw JSON string format.
/// Response contains exchange rate of `cur: Currency` on `date`.
///
/// # Warning
/// Need `date` in ISO 8601 format like `"YYYY-MM-DD"`.
///
/// # Errors
/// Return `Err<String>` if you put `Currency::BYN` in `cur` param.
///
/// # Examples
/// ```
/// # use belrates::*;
/// let json = date_get(&Currency::USD, "2018-04-20").unwrap();
/// ```
fn date_get(cur: &Currency, date: &str) -> Result<String, String> {
    //we can't get the rate of BYN in BYN, that's a silly
    if *cur == Currency::BYN {
        return Err("cannot get currency for BYN".to_string());
    }
    let url = format!("{}{}?onDate={}",URL_RATES, cur.get_id(), date);
    //send request to server
    request(&url)
}

fn request(url: &str) -> Result<String, String> {
    let res = reqwest::get(url);
    let mut res = match res {
        Ok(val) => val,
        Err(_) => return Err("cannot get data from server".to_string()),
    };
    //handling 404
    if res.status() == StatusCode::NOT_FOUND {
        return Err("got 404".to_string())
    }
    match res.text() {
        Ok(val) => Ok(val),
        Err(_) => Err("cannot get from body".to_string())
    }
}

//! Belrates contains useful functions helps you to get actual exhange rate of foreign currencies
//! in Belarusian rouble - national currency of Republic of Belarus.
//!
//! Exchange rates provided by API of site of National Bank of Republic of Belarus
//! [nbrb.by](http://www.nbrb.by).
//!

extern crate hyper;
extern crate futures;

use std::sync::{Mutex, Arc};
use std::u32;
use hyper::{Client, StatusCode};
use hyper::rt::{self, Future, Stream};
use futures::sync::mpsc;
use futures::Async::Ready;

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
pub fn get_from_server(cur: &Currency) -> Result<String, String> {
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
pub fn date_get(cur: &Currency, date: &str) -> Result<String, String> {
    //we can't get the rate of BYN in BYN, that's a silly
    if *cur == Currency::BYN {
        return Err("cannot get currency for BYN".to_string());
    }
    let url = format!("{}{}?onDate={}",URL_RATES, cur.get_id(), date);
    //send request to server
    request(&url)
}

fn request(url: &str) -> Result<String, String> {
    //create a chanel to sending body of response from tokio runtime to main thread
    let (sender, mut receiver) = mpsc::channel::<String>(512);
    //parsing url
    let url = url.parse::<hyper::Uri>().unwrap();
    //packing sender
    let sender = Arc::new(Mutex::new(sender));
    //start the tokio
    rt::run(fetch_url(url, sender));
    //its seems we get some response, converting then to String
    if let Ok(Ready(Some(vals))) = receiver.poll() {
        if vals.starts_with("404") {
            Err("got 404".to_string())
        } else {
            Ok(vals)
        }
    }
    else {
        Err("invalid receiving".to_string())
    }
}

fn fetch_url(url: hyper::Uri, sender: Arc<Mutex<mpsc::Sender<String>>>)
                            -> impl Future<Item=(), Error=()> {
    let client = Client::new();
    //some magic
    client
        .get(url)
        .and_then(move |res| {
            if res.status() == StatusCode::NOT_FOUND {
                eprintln!("got 404");
                let mut paniker = sender.lock().unwrap();
                paniker.try_send("404".to_string()).unwrap()
                //FIXME можно попробовать передать сам статускод есть возможность передать только
                //в данной ветви
            }
            res.into_body().for_each(move |chunk| {
                let mut sender = sender.lock().unwrap();
                sender.try_send(String::from_utf8(chunk.to_vec()).unwrap())
                    .map_err(|e| panic!("expecting channnel is open, error={}", e))
            })
        })
        .map_err(|err| {
            eprintln!("error {}", err);
        })
}

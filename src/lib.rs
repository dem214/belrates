extern crate hyper;
extern crate futures;

use std::sync::{Mutex, Arc};
use std::u32;
use hyper::{Client, Chunk, StatusCode};
use hyper::rt::{self, Future, Stream};
use futures::sync::mpsc;
use futures::Async::Ready;

mod currency;
pub use currency::Currency;

const URL_RATES: &'static str = "http://www.nbrb.by/API/ExRates/Rates/";

#[derive(Debug)]
pub struct Rate {
    pub id: u32,
    pub date: String,
    pub abbreviation: Currency,
    pub scale: u32,
    pub name: String,
    pub rate: f32,
}

impl Rate {
    pub fn from_string(s: String) -> Rate {
        let s = s.trim_left_matches('{').trim_right_matches('}');
        let table: Vec<&str> = s.split(',').collect();

        let row: Vec<&str> = table[0].split(':').collect();
        let id: u32 = row[1].parse().unwrap();

        let row: Vec<&str> = table[1].split(':').collect();
        let date = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[2].split(':').collect();
        let abbreviation = Currency::from_str(&row[1].trim_matches('"').to_string()).unwrap();

        let row: Vec<&str> = table[3].split(':').collect();
        let scale: u32 = row[1].parse().unwrap();

        let row: Vec<&str> = table[4].split(':').collect();
        let name = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[5].split(':').collect();
        let rate: f32 = row[1].parse().unwrap();

        Rate{id, date, abbreviation, scale, name, rate}
    }
    pub fn from_server_today(cur: &currency::Currency) -> Rate {
        Rate::from_string(get_from_server(cur).unwrap())
    }
    ///Return actual rate that represent official rate devided by official currency scale of NBRB.
    ///
    /// `self.act_rate == self.rate / self.scale`
    ///
    ///That means a cost of 1 unit of the foreing currency in the belarusian roubles.
    ///
    ///# Example
    ///
    ///```
    ///let some_rate = Rate {
    ///    # id: 1,
    ///    # date: "today".to_string(),
    ///    # abbreviation: Currency::RUB,
    ///    # name: "Рубль".to_string(),
    ///    // snip..
    ///    scale: 100,
    ///    rate: 3.14,
    ///};
    ///
    ///assert_eq!(some_rate.act_rate(), 0.0314)
    ///```
    pub fn act_rate(&self) -> f32 {
        self.rate / self.scale as f32
    }
}

pub fn get_from_server(cur: &currency::Currency) -> Result<String, String> {
    //we can't get the rate of BYN in BYN, its a silly
    if *cur == Currency::BYN {
        return Err("cannot get currency for BYN".to_string());
    }
    let url = format!("{}{}",URL_RATES, cur.get_id());
    //send request to server
    request(&url)
}
///Need `date` in ISO 8601 format like `"YYYY-MM-DD"`.
pub fn date_get(cur: &currency::Currency, date: &str) -> Result<String, String> {
    //we can't get the rate of BYN in BYN, its a silly
    if *cur == Currency::BYN {
        return Err("cannot get currency for BYN".to_string());
    }
    let url = format!("{}{}?onDate={}",URL_RATES, cur.get_id(), date);
    //send request to server
    request(&url)
}

fn request(url: &str) -> Result<String, String> {
    //create a chanel to sending body of response from tokio runtime to main thread
    let (sender, mut receiver) = mpsc::channel::<Chunk>(512);
    //parsing url
    let url = url.parse::<hyper::Uri>().unwrap();
    //packing sender
    let sender = Arc::new(Mutex::new(sender));
    //start the tokio
    rt::run(fetch_url(url, sender));
    //its seems we get some response, converting then to String
    if let Ok(Ready(Some(vals))) = receiver.poll() {
        Ok(String::from_utf8(vals.to_vec()).unwrap())
    }
    else {
        Err("invalid receiving".to_string())
    }
}

fn fetch_url(url: hyper::Uri, sender: Arc<Mutex<mpsc::Sender<Chunk>>>) -> impl Future<Item=(), Error=()> {
    let client = Client::new();
    //some magic
    client
        .get(url)
        .and_then(move |res| {
            if res.status() == StatusCode::NOT_FOUND {
                eprintln!("got 404");
            }
            res.into_body().for_each(move |chunk| {
                let mut sender = sender.lock().unwrap();
                sender.try_send(chunk)
                    .map_err(|e| panic!("expecting channnel is open, error={}", e))
            })
        })
        .map_err(|err| {
            eprintln!("error {}", err);
        })
}

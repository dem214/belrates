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
    id: u32,
    date: String,
    abbreviation: Currency,
    scale: u32,
    name: String,
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
}

pub fn get_from_server(cur: Currency) -> Result<String, String> {
    if cur == Currency::BYN {
        return Err("cannot get currency for BYN".to_string());
    }

    let (sender, mut receiver) = mpsc::channel::<Chunk>(512);
    let url = format!("{}{}",URL_RATES, cur.get_id());
    let url = url.parse::<hyper::Uri>().unwrap();
    let sender = Arc::new(Mutex::new(sender));

    rt::run(fetch_url(url, sender));

    if let Ok(Ready(Some(vals))) = receiver.poll() {
        Ok(String::from_utf8(vals.to_vec()).unwrap())
    }
    else {
        Err("invalid receiving".to_string())
    }
}

fn fetch_url(url: hyper::Uri, sender: Arc<Mutex<mpsc::Sender<Chunk>>>) -> impl Future<Item=(), Error=()> {
    let client = Client::new();

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

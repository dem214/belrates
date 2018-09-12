extern crate hyper;
extern crate futures;

use std::sync::{Mutex, Arc};
use std::u32;
use hyper::{Client, Chunk};
use hyper::rt::{self, Future, Stream};
use futures::sync::mpsc;
use futures::Async::Ready;

#[derive(Debug)]
struct RateResponced {
    id: u32,
    date: String,
    abbreviation: String,
    scale: u32,
    name: String,
    rate: f32,
}

impl RateResponced {
    fn from_string(s: String) -> RateResponced {
        let s = s.trim_left_matches('{').trim_right_matches('}');
        let table: Vec<&str> = s.split(',').collect();

        let row: Vec<&str> = table[0].split(':').collect();
        let id: u32 = row[1].parse().unwrap();

        let row: Vec<&str> = table[1].split(':').collect();
        let date = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[2].split(':').collect();
        let abbreviation = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[3].split(':').collect();
        let scale: u32 = row[1].parse().unwrap();

        let row: Vec<&str> = table[4].split(':').collect();
        let name = row[1].trim_matches('"').to_string();

        let row: Vec<&str> = table[5].split(':').collect();
        let rate: f32 = row[1].parse().unwrap();

        RateResponced{id, date, abbreviation, scale, name, rate}
    }
}

fn main() {
    let usd = 145;
    let responce = get_from_server(usd).unwrap();
    let rate = RateResponced::from_string(responce);
    println!("{}", rate.rate);


}
fn get_from_server(cur: u32) -> Result<String, String> {
    let (sender, mut receiver) = mpsc::channel::<Chunk>(512);
    let url = format!("http://www.nbrb.by/API/ExRates/Rates/{}", cur);
    let url = url.parse::<hyper::Uri>().unwrap();
    let sender = Arc::new(Mutex::new(sender));

    rt::run(fetch_url(url, sender));

    if let Ok(Ready(Some(vals))) = receiver.poll() {
        Ok(String::from_utf8(vals.to_vec()).unwrap())
    }
    else {
        Err("Invalid receiving".to_string())
    }
}

fn fetch_url(url: hyper::Uri, sender: Arc<Mutex<mpsc::Sender<Chunk>>>) -> impl Future<Item=(), Error=()> {
    let client = Client::new();

    client
        .get(url)
        .and_then(move |res| {
            res.into_body().for_each(move |chunk| {
                let mut sender = sender.lock().unwrap();
                sender.try_send(chunk)
                    .map_err(|e| panic!("example expects stdout is open, error={}", e))
            })
        //.map(|_| {
        //    println!("Done");
        //})
        })
        .map_err(|err| {
            eprintln!("Error {}", err);
        })
}

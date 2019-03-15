use belrates::Rate;
use belrates::Currency;

fn main() {
    let usd_rate = Rate::from_server_today(&Currency::USD).unwrap();
    println!("Price of 1 {} on date {} is {}", usd_rate.name, usd_rate.date, usd_rate.act_rate());
}
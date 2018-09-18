
extern crate belrates;
use belrates::*;

fn main() {
    let usd_rate = Rate::from_server_today(&Currency::USD);
    println!("Price of 1 {} is {}", usd_rate.name, usd_rate.act_rate());
}

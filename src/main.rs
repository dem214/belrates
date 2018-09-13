
extern crate belrates;
use belrates::*;

fn main() {
    let responce = get_from_server(Currency::USD).unwrap();
    let rate = Rate::from_string(responce);
    println!("{}", rate.rate);
}

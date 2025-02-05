use go_duration::GoDuration;
// `GoDuration::from_str` requires `std::str::FromStr`
use std::str::FromStr;

fn main() {
    let dur = GoDuration(0);
    println!("{}", dur.nanoseconds()); // => "0"
    println!("{}", dur); // => "0s"

    let dur: GoDuration = 42.into();
    println!("{}", dur.nanoseconds()); // => "42"
    println!("{}", dur); // => "42ns"

    let dur = str::parse::<GoDuration>("0s");
    println!("{:?}", dur); // => "Ok(GoDuration(0))"
    println!("{}", dur.unwrap()); // => "0s"

    let dur = GoDuration::from_str("4000ns");
    println!("{:?}", dur); // => "Ok(GoDuration(4000))"
    println!("{}", dur.unwrap()); // => "4µs"

    let dur: Result<GoDuration, _> = "60m".try_into();
    println!("{:?}", dur); // => "Ok(GoDuration(3600000000000))"
    println!("{}", dur.unwrap()); // => "1h"
}

use go_duration::GoDuration;
use nom::{
    bytes::complete::{tag, take_till},
    combinator::opt,
    multi::fold_many1,
    sequence::terminated,
    Parser,
};

fn main() {
    // Basic usage
    let input = "1210ms";
    let (_, dur) = go_duration::nom::go_duration.parse(input).unwrap();
    println!("{:?}", dur);  // => GoDuration(1210000000)
    println!("{}", dur);    // => 1.21s

    // Using with other nom combinators
    let input = "10ns 30ms 1m 1m30s";
    let (_, arr) = fold_many1(
        terminated(take_till(|c: char| c.is_ascii_whitespace()), opt(tag(" ")))
            .and_then(go_duration::nom::go_duration),
        Vec::new,
        |mut arr, dur: GoDuration| {
            arr.push(dur);
            arr
        },
    )
    .parse(input)
    .unwrap();
    /*
     * GoDuration(10)
     * GoDuration(30000000)
     * GoDuration(60000000000)
     * GoDuration(90000000000)
     */
    arr.iter().for_each(|dur| println!("{dur:?}"));
}
# go-duration

go-duration is parsing and formatting library for Go-lang style `time.Duration`.

<!-- toc -->

- [Usage](#usage)
- [Examples](#examples)
  - [Basic](#basic)
  - [Serialization/Deserialization](#serializationdeserialization)
  - [Nom parser](#nom-parser)

<!-- tocstop -->

## Usage

Add `go_duration` as a dependency in Cargo.toml

```toml
[dependencies]
go_duration = "<version>"

# enable `serde` feature to use serialization/deserialization via serde
go_duration = { version = "<version>", features = ["serde"] }
```

## Examples

### Basic

```rust
use go_duration::GoDuration;
// `GoDuration::from_str` requires `std::str::FromStr`
use std::str::FromStr;

fn main() {
    let dur = GoDuration(0);
    println!("{}", dur.nanoseconds());  // => "0"
    println!("{}", dur);                // => "0s"

    let dur: GoDuration = 42.into();
    println!("{}", dur.nanoseconds());  // => "42"
    println!("{}", dur);                // => "42ns"

    let dur = str::parse::<GoDuration>("0s");
    println!("{:?}", dur);              // => "Ok(GoDuration(0))"
    println!("{}", dur.unwrap());       // => "0s"

    let dur = GoDuration::from_str("4000ns");
    println!("{:?}", dur);              // => "Ok(GoDuration(4000))"
    println!("{}", dur.unwrap());       // => "4µs"

    let dur: Result<GoDuration, _> = "60m".try_into();
    println!("{:?}", dur);              // => "Ok(GoDuration(3600000000000))"
    println!("{}", dur.unwrap());       // => "1h"
}
```

### Serialization/Deserialization

```rust
use go_duration::GoDuration;
use serde::{Serialize, Deserialize};

fn main() {
    #[derive(Debug, Serialize, Deserialize)]
    struct DurationTest {
        pub duration: GoDuration,
    }

    let input = r#"{"duration":"90s"}"#;
    let dur_test: DurationTest = serde_json::from_str(input).unwrap();
    let output = serde_json::to_string(&dur_test).unwrap();
    let expected = r#"{"duration":"1m30s"}"#;
    assert_eq!(expected, output);

    #[derive(Debug, Serialize, Deserialize)]
    struct DurationNanosTest {
        #[serde(with = "go_duration::serde::nanoseconds")]
        pub duration: GoDuration,
    }

    let input = r#"{"duration":9000000}"#;
    let dur_test: DurationNanosTest = serde_json::from_str(input).unwrap();
    let output = serde_json::to_string(&dur_test).unwrap();
    let expected = r#"{"duration":9000000}"#;
    assert_eq!(expected, output);
}
```

### Nom parser

```rust
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
```

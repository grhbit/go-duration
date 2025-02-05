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
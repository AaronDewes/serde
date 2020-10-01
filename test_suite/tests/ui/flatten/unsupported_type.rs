use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Outer {
    outer: String,
    #[serde(flatten)]
    inner: String,
}

fn main() {}

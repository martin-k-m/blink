use serde::Serialize;

#[derive(Serialize)]
struct Greeting {
    message: String,
}

fn main() {
    let greeting = Greeting {
        message: "hello from the rust-crate example".to_string(),
    };
    println!("{}", serde_json::to_string_pretty(&greeting).unwrap());
}

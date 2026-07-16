use serde::Serialize;

#[derive(Serialize)]
struct Fixture {
    name: &'static str,
}

fn main() {
    let fixture = Fixture {
        name: "rust-project-fixture",
    };
    println!("{}", fixture.name);
}

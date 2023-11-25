use ic_cdk::{query, export_candid};

#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

export_candid!();
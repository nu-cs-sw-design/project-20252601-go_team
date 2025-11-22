use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();

    let team = String::from("blue");
    let score = 3;

    scores.insert(team, score);
    for (key, value) in &scores {
        println!("{key}, {value}");
    }

    println!("done");
}
use std::{env, fs};

fn main() {
    let home_dir = env::var("HOME").expect("HOME Environment Variable not found");

    println!("{}", home_dir);
}

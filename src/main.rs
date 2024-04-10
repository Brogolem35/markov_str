use std::{
    collections::HashMap,
    env,
    fs::{self, read_to_string},
};

struct ChainItem {
    items: Vec<(String, u32)>,
    totalcnt: u32,
}

fn main() {
    let home_dir = env::var("HOME").expect("HOME Environment Variable not found");

    let paths = fs::read_dir(&home_dir).expect(&format!("Can't read files from: {}", home_dir));

    // Only the files remain
    let files = paths
        .filter_map(|f| f.ok())
        .filter(|f| match f.file_type() {
            Err(_) => false,
            Ok(f) => f.is_file(),
        });

    let strings = files.filter_map(|f| read_to_string(f.path()).ok());

    for s in strings {
        println!("{}", s);
    }

    println!("{}", home_dir);
}

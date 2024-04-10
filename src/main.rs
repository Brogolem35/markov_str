use std::{env, fs};

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

    for file in files {
        println!("Name: {}", file.path().display())
    }

    println!("{}", home_dir);
}

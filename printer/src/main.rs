use std::env::args;
use std::fs;

fn main() {
    let file_name = args().nth(1).expect("provide a file path");
    for b in fs::read(file_name).expect("reading failed").iter() {
        print!("{:b} ", b);
    }
    print!("\n");
}

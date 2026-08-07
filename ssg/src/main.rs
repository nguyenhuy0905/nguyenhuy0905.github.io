#![allow(dead_code)]
#![allow(unused)]
mod exe;
mod block;
use exe::Runner;

fn main() {
    let mut exe = Runner::new();
    let mut out: Vec<u8> = Vec::new();
    exe.run("<h1>{{ yield hello; }}</h1>".as_bytes(), &mut out);
    println!("Output:\n{}", String::from_utf8(out).unwrap());
    // println!("Hello, world!");
}

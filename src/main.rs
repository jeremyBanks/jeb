use jeb::{calculate, greet, process_data};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let name = &args[1];
        println!("{}", greet(name));
    } else {
        println!("{}", greet("World"));
    }

    // Demo the other functions
    println!("Calculate 5 + 3 = {}", calculate(5, 3));

    let data = vec![1, 2, 3, 4, 5];
    let processed = process_data(&data);
    println!("Processed data: {:?}", processed);
}

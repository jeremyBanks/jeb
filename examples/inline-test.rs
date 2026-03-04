// Testing the inline crate - self-modifying code
// Run with: cargo run --manifest-path /Users/matte/jeb/Cargo.toml --example inline-test

use inline::cell;

fn main() {
    let mut heartbeat_count = cell(0u32);
    
    println!("Heartbeat #{}", *heartbeat_count + 1);
    println!("This value persists across runs!");
    
    heartbeat_count.value = *heartbeat_count + 1;
    // The source file has now been updated with the new value!
    
    println!("Updated count written back to source file.");
}

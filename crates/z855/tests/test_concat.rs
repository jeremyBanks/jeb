#[cfg(test)]
mod test {
    use std::fs;
    
    #[test]
    fn test_gap_rand_both_modes() {
        let input = fs::read("test-cases/gap-rand-1-8-5-4.input").unwrap();
        
        // Non-concatenatable
        let normal = z855::encode(&input);
        println!("Normal:        {}", normal);
        println!("Length: {} mod 5: {}", normal.len(), normal.len() % 5);
        
        let decoded_normal = z855::decode(&normal).unwrap();
        assert_eq!(decoded_normal, input);
        println!("Normal decode: OK");
        
        // Concatenatable  
        let opts = z855::Z855Options {
            concatenatable: true,
            ..Default::default()
        };
        let concat = z855::encode_with_options(&input, &opts).unwrap();
        println!("\nConcat:        {}", concat);
        println!("Length: {} mod 5: {}", concat.len(), concat.len() % 5);
        
        // Find where they differ
        for (i, (c1, c2)) in normal.chars().zip(concat.chars()).enumerate() {
            if c1 != c2 {
                println!("First diff at position {}: '{}' vs '{}'", i, c1, c2);
                break;
            }
        }
        if concat.len() != normal.len() {
            println!("Length difference: {} vs {}", normal.len(), concat.len());
            println!("Extra in concat: {:?}", &concat[normal.len()..]);
        }
    }
}

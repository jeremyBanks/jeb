#[cfg(test)]
mod test {
    use std::fs;
    
    #[test]
    fn test_concatenatable_roundtrip() {
        let input = fs::read("test-cases/gap-rand-1-8-5-4.input").unwrap();
        
        let opts = z855::Z855Options {
            concatenatable: true,
            ..Default::default()
        };
        
        let encoded = z855::encode_with_options(&input, &opts).unwrap();
        println!("Encoded: {}", encoded);
        println!("Length: {} mod 5: {}", encoded.len(), encoded.len() % 5);
        
        let decoded = z855::decode(&encoded).unwrap();
        assert_eq!(decoded, input, "Round-trip failed!");
        println!("SUCCESS!");
    }
}

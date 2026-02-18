#[cfg(test)]
mod test {
    use std::fs;
    
    #[test]
    fn test_concatenatable_roundtrip_single() {
        let input = fs::read("test-cases/gap-rand-1-8-5-4.input").unwrap();
        let opts = z855::Z855Options {
            concatenatable: true,
            ..Default::default()
        };
        let encoded = z855::encode_with_options(&input, &opts).unwrap();
        assert_eq!(encoded.len() % 5, 0, "Length must be divisible by 5");
        let decoded = z855::decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }
    
    #[test]
    fn test_concatenatable_all_test_cases() {
        let opts = z855::Z855Options {
            concatenatable: true,
            ..Default::default()
        };
        
        let mut passed = 0;
        let mut failed = 0;
        
        for entry in fs::read_dir("test-cases").unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".input") { continue; }
            
            let input = fs::read(entry.path()).unwrap();
            // Skip error test cases
            if input.starts_with(b"<error") { continue; }
            
            let encoded = z855::encode_with_options(&input, &opts).unwrap();
            
            if encoded.len() % 5 != 0 {
                eprintln!("FAIL {}: length {} not divisible by 5", name, encoded.len());
                failed += 1;
                continue;
            }
            
            match z855::decode(&encoded) {
                Ok(decoded) => {
                    if decoded != input {
                        eprintln!("FAIL {}: round-trip mismatch", name);
                        failed += 1;
                    } else {
                        passed += 1;
                    }
                }
                Err(e) => {
                    eprintln!("FAIL {}: decode error {:?} (encoded: {})", name, e, 
                        &encoded[..encoded.len().min(40)]);
                    failed += 1;
                }
            }
        }
        
        println!("{} passed, {} failed", passed, failed);
        assert_eq!(failed, 0, "Some test cases failed concatenatable round-trip");
    }
}

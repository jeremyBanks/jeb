use wasm_bindgen::prelude::*;

/// A simple greeting function that returns a formatted message.
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Process some data - placeholder implementation.
pub fn process_data(data: &[u8]) -> Vec<u8> {
    // Placeholder: just returns a copy of the input
    data.to_vec()
}

/// Calculate something - placeholder implementation.
pub fn calculate(x: i32, y: i32) -> i32 {
    // Placeholder: simple addition
    x + y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_greet_empty() {
        let result = greet("");
        assert_eq!(result, "Hello, !");
    }

    #[test]
    fn test_process_data() {
        let data = vec![1, 2, 3, 4, 5];
        let result = process_data(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_process_data_empty() {
        let data: Vec<u8> = vec![];
        let result = process_data(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_calculate() {
        assert_eq!(calculate(2, 3), 5);
        assert_eq!(calculate(-1, 1), 0);
        assert_eq!(calculate(0, 0), 0);
    }
}

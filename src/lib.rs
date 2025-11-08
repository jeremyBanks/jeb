use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use wasm_bindgen::prelude::*;

/// A simple data structure to demonstrate serde serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub id: i64,
    pub name: String,
    pub metadata: Option<String>,
}

impl Entity {
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// A simple greeting function that returns a formatted message.
#[wasm_bindgen]
#[instrument]
pub fn greet(name: &str) -> String {
    info!("Greeting user: {}", name);
    format!("Hello, {}!", name)
}

/// Process some data - placeholder implementation.
#[instrument(skip(data))]
pub fn process_data(data: &[u8]) -> Vec<u8> {
    debug!("Processing {} bytes of data", data.len());
    // Placeholder: just returns a copy of the input
    data.to_vec()
}

/// Calculate something - placeholder implementation.
#[instrument]
pub fn calculate(x: i32, y: i32) -> i32 {
    debug!("Calculating: {} + {}", x, y);
    // Placeholder: simple addition
    x + y
}

/// Serialize an Entity to JSON.
#[instrument(skip(entity))]
pub fn entity_to_json(entity: &Entity) -> Result<String, serde_json::Error> {
    info!("Serializing entity with id: {}", entity.id);
    serde_json::to_string(entity)
}

/// Deserialize an Entity from JSON.
#[instrument(skip(json))]
pub fn entity_from_json(json: &str) -> Result<Entity, serde_json::Error> {
    debug!("Deserializing entity from JSON");
    serde_json::from_str(json)
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

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(1, "Test Entity".to_string());
        assert_eq!(entity.id, 1);
        assert_eq!(entity.name, "Test Entity");
        assert_eq!(entity.metadata, None);
    }

    #[test]
    fn test_entity_with_metadata() {
        let entity = Entity::new(1, "Test".to_string())
            .with_metadata("Some metadata".to_string());
        assert_eq!(entity.metadata, Some("Some metadata".to_string()));
    }

    #[test]
    fn test_entity_serialization() {
        let entity = Entity::new(42, "John Doe".to_string());
        let json = entity_to_json(&entity).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"name\":\"John Doe\""));
    }

    #[test]
    fn test_entity_deserialization() {
        let json = r#"{"id":123,"name":"Jane Doe","metadata":null}"#;
        let entity = entity_from_json(json).unwrap();
        assert_eq!(entity.id, 123);
        assert_eq!(entity.name, "Jane Doe");
        assert_eq!(entity.metadata, None);
    }

    #[test]
    fn test_entity_round_trip() {
        let original = Entity::new(999, "Round Trip".to_string())
            .with_metadata("test metadata".to_string());
        let json = entity_to_json(&original).unwrap();
        let deserialized = entity_from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn parse_payload<T>(handler_payload: &HashMap<String, Value>) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    // First, serialize the HashMap to a JSON Value
    let json_value = serde_json::to_value(handler_payload)?;

    // Then, deserialize the JSON Value to our target type
    serde_json::from_value(json_value)
}

pub fn parse_payload_str<T>(json_payload: &str) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(json_payload)
}

pub fn parse_payload_value<T>(json_payload: Value) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(json_payload)
}

// Example struct
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTemplateId {
    pub room_id: String,
    pub template_id: String,
    pub user_id: String,
    pub images: Vec<String>,
}
pub fn json_to_hashmap(json_payload: &str) -> Result<HashMap<String, Value>, serde_json::Error> {
    serde_json::from_str(json_payload)
}

pub fn value_to_hashmap(json_payload: Value) -> Result<HashMap<String, Value>, serde_json::Error> {
    serde_json::from_value(json_payload)
}

// Usage example
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template_id = "123";
    let json_payload = format!(
        r#"
            {{
                "roomId": "room123",
                "templateId": "{template_id}",
                "userId": "user789",
                "images": ["https://example.com/image1.jpg", "https://example.com/image2.jpg"]
            }}
            "#
    );

    // Parse the payload
    match parse_payload_str::<ChangeTemplateId>(&json_payload) {
        Ok(change_template) => println!("Parsed ChangeTemplateId: {change_template:?}"),
        Err(e) => println!("Error parsing payload: {e}"),
    }
    let json_payload_value = json!({
        "roomId": "room123",
        "templateId": template_id,
        "userId": "user789",
        "images": ["https://example.com/image1.jpg", "https://example.com/image2.jpg"]
    });

    // Parse the payload
    match parse_payload_value::<ChangeTemplateId>(json_payload_value.clone()) {
        Ok(change_template) => println!("Parsed ChangeTemplateId: {change_template:?}"),
        Err(e) => println!("Error parsing payload: {e}"),
    }

    // Parse the payload
    match parse_payload_str::<ChangeTemplateId>(&json_payload) {
        Ok(change_template) => println!("Parsed ChangeTemplateId: {change_template:?}"),
        Err(e) => println!("Error parsing payload: {e}"),
    }

    let another_hash_map = value_to_hashmap(json_payload_value)?;
    println!("Parsed HashMap: {another_hash_map:?}");
    // Parse JSON to HashMap
    let hashmap = json_to_hashmap(&json_payload)?;
    println!("Parsed HashMap: {hashmap:?}");

    // Parse the payload
    match parse_payload::<ChangeTemplateId>(&hashmap) {
        Ok(change_template) => println!("Parsed ChangeTemplateId: {change_template:?}"),
        Err(e) => println!("Error parsing payload: {e}"),
    }

    Ok(())
}

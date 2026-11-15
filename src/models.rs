use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: Value, // Supports both String and multimodal arrays
}

#[derive(Serialize, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedData {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub messages: Vec<Message>,
}

impl Default for SavedData {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/chat/completions".to_owned(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_owned(),
            messages: vec![Message {
                role: "system".to_owned(),
                content: Value::String("You are Custard, a lightweight AI assistant built in Rust.".to_owned()),
            }],
        }
    }
}
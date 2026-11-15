use crate::models::{ChatRequest, ChatResponse, Message};

pub fn send_chat_request(
    url: &str,
    api_key: &str,
    model: &str,
    messages: &[Message],
) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let body = ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
    };

    let res = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json::<ChatResponse>());

    match res {
        Ok(data) => {
            if let Some(choice) = data.choices.first() {
                if let Some(text) = choice.message.content.as_str() {
                    Ok(text.to_string())
                } else {
                    Ok(choice.message.content.to_string())
                }
            } else {
                Err("No response choices returned from API.".to_string())
            }
        }
        Err(e) => Err(format!("API request failed: {}", e)),
    }
}
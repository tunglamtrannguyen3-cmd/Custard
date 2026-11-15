use custard::api::send_chat_request;
use custard::models::{ChatRequest, ChatResponse, Message, SavedData};
use custard::storage::storage_path;
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

struct MockRequest {
    request_line: String,
    headers: HashMap<String, String>,
    body: String,
}

fn mock_server(status: &str, response_body: &str) -> (String, thread::JoinHandle<MockRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind mock server");
    let address = listener
        .local_addr()
        .expect("Failed to get mock server address");
    let status = status.to_owned();
    let response_body = response_body.to_owned();

    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("Failed to accept API request");
        read_request(stream, &status, &response_body)
    });

    (format!("http://{address}"), server)
}

fn read_request(stream: TcpStream, status: &str, response_body: &str) -> MockRequest {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("Failed to read request line");

    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read headers");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
        }
    }

    let content_length = headers
        .get("content-length")
        .expect("Request is missing Content-Length")
        .parse::<usize>()
        .expect("Invalid Content-Length");
    let mut body = vec![0; content_length];
    reader
        .read_exact(&mut body)
        .expect("Failed to read request body");

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
        response_body.len()
    );
    reader
        .get_mut()
        .write_all(response.as_bytes())
        .expect("Failed to write mock response");

    MockRequest {
        request_line: request_line.trim().to_owned(),
        headers,
        body: String::from_utf8(body).expect("Request body was not valid UTF-8"),
    }
}

#[test]
fn test_default_saved_data() {
    let data = SavedData::default();
    assert_eq!(data.api_url, "https://api.openai.com/v1/chat/completions");
    assert_eq!(data.model, "gpt-4o-mini");
    assert!(data.api_key.is_empty());
    assert_eq!(data.messages.len(), 1);
    assert_eq!(data.messages[0].role, "system");
}

#[test]
fn test_message_serialization() {
    let msg = Message {
        role: "user".to_owned(),
        content: json!("Hello Custard!"),
    };

    let json_str = serde_json::to_string(&msg).expect("Failed to serialize message");
    let value: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse serialized message");
    assert_eq!(value["role"], "user");
    assert_eq!(value["content"], "Hello Custard!");

    let deserialized: Message = serde_json::from_str(&json_str).expect("Failed to deserialize message");
    assert_eq!(deserialized.role, "user");
    assert_eq!(deserialized.content, json!("Hello Custard!"));
}

#[test]
fn test_chat_request_payload() {
    let messages = vec![
        Message {
            role: "system".to_owned(),
            content: json!("You are a test assistant."),
        },
        Message {
            role: "user".to_owned(),
            content: json!("Ping"),
        },
    ];

    let req = ChatRequest {
        model: "gpt-4o-mini".to_owned(),
        messages,
    };

    let json_str = serde_json::to_string(&req).expect("Failed to serialize ChatRequest");
    let value: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse serialized ChatRequest");
    assert_eq!(value["model"], "gpt-4o-mini");
    assert_eq!(value["messages"][1]["content"], "Ping");
}

#[test]
fn test_chat_response_deserialization() {
    let json_data = r#"{
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "Pong from AI"
                }
            }
        ]
    }"#;

    let res: ChatResponse = serde_json::from_str(json_data).expect("Failed to parse ChatResponse");
    assert_eq!(res.choices.len(), 1);
    assert_eq!(res.choices[0].message.role, "assistant");
    assert_eq!(res.choices[0].message.content, json!("Pong from AI"));
}

#[test]
fn test_storage_path_resolution() {
    if let Some(path) = storage_path() {
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("custard"));
        assert!(path_str.ends_with("data.json"));
    } else {
        panic!("Storage path could not be resolved on this system");
    }
}

#[test]
fn test_saved_data_json_roundtrip() {
    let original = SavedData {
        api_url: "http://localhost:11434/v1/chat/completions".to_owned(),
        api_key: "ollama-dummy-key".to_owned(),
        model: "llama3".to_owned(),
        messages: vec![
            Message {
                role: "system".to_owned(),
                content: json!("System instructions"),
            },
            Message {
                role: "user".to_owned(),
                content: json!("Hello!"),
            },
        ],
    };

    let json_str = serde_json::to_string_pretty(&original).expect("Failed to serialize SavedData");
    let loaded: SavedData = serde_json::from_str(&json_str).expect("Failed to deserialize SavedData");

    assert_eq!(loaded.api_url, original.api_url);
    assert_eq!(loaded.api_key, original.api_key);
    assert_eq!(loaded.model, original.model);
    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.messages[1].content, json!("Hello!"));
}

#[test]
fn test_send_chat_request_posts_payload_and_returns_reply() {
    let response = r#"{"choices":[{"message":{"role":"assistant","content":"Pong"}}]}"#;
    let (url, server) = mock_server("200 OK", response);
    let messages = vec![Message {
        role: "user".to_owned(),
        content: json!("Ping"),
    }];

    let reply = send_chat_request(
        &format!("{url}/chat/completions"),
        "test-key",
        "test-model",
        &messages,
    )
    .expect("API request should succeed");
    assert_eq!(reply, "Pong");

    let request = server.join().expect("Mock server failed");
    assert_eq!(request.request_line, "POST /chat/completions HTTP/1.1");
    assert_eq!(
        request.headers.get("authorization").map(String::as_str),
        Some("Bearer test-key")
    );

    let body: serde_json::Value =
        serde_json::from_str(&request.body).expect("Failed to parse request JSON");
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["messages"][0]["content"], "Ping");
}

#[test]
fn test_send_chat_request_reports_empty_choices() {
    let (url, server) = mock_server("200 OK", r#"{"choices":[]}"#);

    let error = send_chat_request(&url, "", "test-model", &[])
        .expect_err("An empty choices list should be rejected");

    assert_eq!(error, "No response choices returned from API.");
    server.join().expect("Mock server failed");
}

#[test]
fn test_send_chat_request_reports_http_errors() {
    let (url, server) = mock_server("500 Internal Server Error", r#"{"error":"unavailable"}"#);

    let error = send_chat_request(&url, "", "test-model", &[])
        .expect_err("A non-success HTTP response should be rejected");

    assert!(error.starts_with("API request failed:"));
    server.join().expect("Mock server failed");
}
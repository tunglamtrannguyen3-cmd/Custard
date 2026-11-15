use crate::api::send_chat_request;
use crate::models::Message;
use crate::storage::{load_from_disk, save_to_disk};
use fltk::{
    app,
    button::Button,
    enums::Font,
    group::{Flex, FlexType},
    input::{Input, SecretInput},
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
};
use std::thread;

#[derive(Clone)]
enum MessageAction {
    ApiResult(Result<String, String>),
}

pub fn run_app() {
    let data = load_from_disk();

    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::default()
        .with_size(900, 600)
        .with_label("Custard - Lightweight AI Chat");

    // Main horizontal layout (Sidebar | Chat Area)
    let mut outer_flex = Flex::default()
        .with_size(900, 600)
        .with_type(FlexType::Row);
    outer_flex.set_margin(10);
    outer_flex.set_spacing(10);

    // --- LEFT SIDEBAR: SETTINGS ---
    let mut sidebar = Flex::default()
        .with_size(260, 600)
        .with_type(FlexType::Column);
    sidebar.set_spacing(8);

    fltk::frame::Frame::default()
        .with_label("🍮 Custard Settings")
        .with_align(fltk::enums::Align::Center | fltk::enums::Align::Inside);

    fltk::frame::Frame::default().with_label("API Endpoint URL:");
    let mut url_input = Input::default();
    url_input.set_value(&data.api_url);

    fltk::frame::Frame::default().with_label("API Key:");
    let mut key_input = SecretInput::default();
    key_input.set_value(&data.api_key);

    fltk::frame::Frame::default().with_label("Model:");
    let mut model_input = Input::default();
    model_input.set_value(&data.model);

    let mut clear_btn = Button::default().with_label("Clear Chat History");
    
    sidebar.end();
    outer_flex.fixed(&sidebar, 260);

    // --- RIGHT PANEL: CHAT AREA ---
    let mut chat_flex = Flex::default()
        .with_size(620, 600)
        .with_type(FlexType::Column);
    chat_flex.set_spacing(8);

    // Chat display buffer
    let mut chat_display = TextDisplay::default();
    let chat_buffer = TextBuffer::default();
    chat_display.set_buffer(chat_buffer.clone());
    chat_display.set_text_font(Font::Courier);
    chat_display.set_text_size(14);

    // Helper to refresh chat display text (marked mut)
    let mut update_display = {
        let mut buf = chat_buffer.clone();
        move |msgs: &[Message]| {
            let mut text = String::new();
            for msg in msgs {
                if msg.role == "system" && !msg.content.starts_with("⚠️") {
                    continue;
                }
                let prefix = match msg.role.as_str() {
                    "user" => "🧑 You:\n",
                    "assistant" => "🍮 Custard:\n",
                    _ => "",
                };
                text.push_str(&format!("{}{}\n\n", prefix, msg.content));
            }
            buf.set_text(&text);
        }
    };

    update_display(&data.messages);

    // Bottom input row (Text input + Send button)
    let mut input_row = Flex::default()
        .with_size(620, 40)
        .with_type(FlexType::Row);
    input_row.set_spacing(6);

    let msg_input = Input::default();
    let mut send_btn = Button::default().with_label("Send"); // marked mut
    input_row.fixed(&send_btn, 80);
    input_row.end();

    chat_flex.end();
    outer_flex.end();
    wind.end();
    wind.make_resizable(true);
    wind.show();

    // Setup channel for background thread communication
    let (s, r) = app::channel::<MessageAction>();

    // Save inputs on change
    let mut data_clone = data.clone();
    url_input.set_callback(move |i| {
        data_clone.api_url = i.value();
        save_to_disk(&data_clone);
    });

    let mut data_clone2 = data.clone();
    key_input.set_callback(move |i| {
        data_clone2.api_key = i.value();
        save_to_disk(&data_clone2);
    });

    let mut data_clone3 = data.clone();
    model_input.set_callback(move |i| {
        data_clone3.model = i.value();
        save_to_disk(&data_clone3);
    });

    // Clear chat callback
    let mut data_clone4 = data.clone();
    let mut disp_ref = update_display.clone();
    clear_btn.set_callback(move |_| {
        data_clone4.messages.truncate(1);
        save_to_disk(&data_clone4);
        disp_ref(&data_clone4.messages);
    });

    // Send message logic
    let send_action = {
        let mut msg_input = msg_input.clone();
        let url_in = url_input.clone();
        let key_in = key_input.clone();
        let model_in = model_input.clone();
        let mut data = data.clone();
        let mut disp_ref2 = update_display.clone();
        let sender = s;

        move || {
            let text = msg_input.value().trim().to_string();
            if text.is_empty() {
                return;
            }

            data.api_url = url_in.value();
            data.api_key = key_in.value();
            data.model = model_in.value();

            data.messages.push(Message {
                role: "user".to_owned(),
                content: text,
            });
            msg_input.set_value("");
            save_to_disk(&data);
            disp_ref2(&data.messages);

            let url = data.api_url.clone();
            let key = data.api_key.clone();
            let model = data.model.clone();
            let history = data.messages.clone();

            thread::spawn(move || {
                let res = send_chat_request(&url, &key, &model, &history);
                let _ = sender.send(MessageAction::ApiResult(res));
            });
        }
    };

    let mut send_clone = send_action; // marked mut
    send_btn.set_callback(move |_| {
        send_clone();
    });

    // Main FLTK event loop
    let mut active_data = data;
    while app.wait() {
        if let Some(action) = r.recv() {
            match action {
                MessageAction::ApiResult(result) => {
                    match result {
                        Ok(reply) => {
                            active_data.messages.push(Message {
                                role: "assistant".to_owned(),
                                content: reply,
                            });
                        }
                        Err(err) => {
                            active_data.messages.push(Message {
                                role: "system".to_owned(),
                                content: format!("⚠️ {}", err),
                            });
                        }
                    }
                    save_to_disk(&active_data);
                    update_display(&active_data.messages);
                }
            }
        }
    }
}
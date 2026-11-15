use crate::api::send_chat_request;
use crate::models::Message;
use crate::storage::{load_from_disk, save_to_disk};
use fltk::{
    app,
    button::Button,
    dialog,
    enums::{Color, Font},
    group::{Flex, FlexType},
    input::{Input, SecretInput},
    menu::Choice as MenuChoice,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
};
use serde_json::json;
use std::fs;
use std::thread;

#[derive(Clone, Copy)]
struct ThemePalette {
    bg: Color,
    panel: Color,
    input: Color,
    text: Color,
    muted: Color,
    border: Color,
    accent: Color,
    button: Color,
    button_text: Color,
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r, g, b)
}

// Ten Ghibli × Touhou-inspired Custard themes.
fn palette_for(index: i32) -> ThemePalette {
    let (bg, panel, input, text, muted, border, accent, button, button_text) =
        match index {
            // Hakurei Serenity — Reimu
            0 => (
                (30, 30, 46), (24, 24, 37), (49, 50, 68),
                (197, 206, 229), (148, 155, 181), (69, 72, 94),
                (198, 160, 246), (69, 72, 94), (245, 245, 250),
            ),

            // Spring of the Valley — Ghibli countryside
            1 => (
                (245, 239, 221), (235, 226, 204), (255, 249, 235),
                (75, 83, 65), (133, 135, 108), (195, 183, 151),
                (111, 153, 105), (111, 153, 105), (255, 255, 245),
            ),

            // Eternal Night — Eientei
            2 => (
                (22, 24, 42), (30, 33, 56), (39, 43, 69),
                (211, 211, 235), (143, 148, 180), (64, 68, 104),
                (170, 150, 220), (75, 67, 116), (245, 240, 255),
            ),

            // Grave of the Fireflies
            3 => (
                (35, 29, 33), (48, 37, 40), (59, 46, 48),
                (232, 211, 197), (169, 143, 132), (93, 72, 73),
                (224, 147, 126), (111, 66, 63), (255, 241, 225),
            ),

            // The Sea of Ponyo
            4 => (
                (24, 48, 66), (29, 66, 85), (39, 82, 103),
                (221, 239, 235), (151, 190, 195), (67, 112, 125),
                (115, 206, 209), (54, 130, 145), (245, 255, 255),
            ),

            // Forest of Magic — Marisa
            5 => (
                (29, 35, 31), (39, 49, 40), (49, 61, 48),
                (220, 226, 195), (154, 171, 139), (75, 93, 68),
                (221, 190, 104), (111, 101, 57), (255, 250, 220),
            ),

            // Forest Spirit — Princess Mononoke
            6 => (
                (31, 42, 36), (42, 56, 46), (52, 69, 55),
                (220, 229, 207), (157, 178, 151), (78, 102, 81),
                (158, 190, 126), (78, 112, 82), (248, 255, 235),
            ),

            // Howl's Golden Autumn
            7 => (
                (48, 37, 35), (64, 48, 43), (77, 58, 49),
                (242, 221, 184), (183, 155, 125), (112, 83, 63),
                (225, 166, 91), (133, 87, 51), (255, 246, 225),
            ),

            // Hakugyokurou — Yuyuko
            8 => (
                (37, 34, 52), (51, 45, 68), (64, 56, 83),
                (233, 220, 242), (177, 158, 194), (96, 80, 119),
                (221, 164, 204), (116, 83, 130), (255, 242, 255),
            ),

            // Kiki's Afternoon
            _ => (
                (42, 43, 56), (55, 57, 70), (67, 70, 84),
                (232, 224, 214), (177, 168, 157), (94, 94, 110),
                (204, 139, 144), (128, 76, 91), (255, 245, 239),
            ),
        };

    ThemePalette {
        bg: rgb(bg.0, bg.1, bg.2),
        panel: rgb(panel.0, panel.1, panel.2),
        input: rgb(input.0, input.1, input.2),
        text: rgb(text.0, text.1, text.2),
        muted: rgb(muted.0, muted.1, muted.2),
        border: rgb(border.0, border.1, border.2),
        accent: rgb(accent.0, accent.1, accent.2),
        button: rgb(button.0, button.1, button.2),
        button_text: rgb(button_text.0, button_text.1, button_text.2),
    }
}

#[derive(Clone)]
enum MessageAction {
    ApiResult(Result<String, String>),
}

pub fn run_app() {
    let data = load_from_disk();

    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::default()
        .with_size(950, 650)
        .with_label("Custard - Lightweight AI Chat");

    // Main horizontal layout (Sidebar | Chat Area)
    let mut outer_flex = Flex::default()
        .with_size(950, 650)
        .with_type(FlexType::Row);
    outer_flex.set_margin(10);
    outer_flex.set_spacing(10);

    // --- LEFT SIDEBAR: SETTINGS & THEMES ---
    let mut sidebar = Flex::default()
        .with_size(260, 650)
        .with_type(FlexType::Column);
    sidebar.set_spacing(8);

    let sidebar_title = fltk::frame::Frame::default()
        .with_label("🍮 Custard Settings")
        .with_align(fltk::enums::Align::Center | fltk::enums::Align::Inside);

    let theme_label = fltk::frame::Frame::default().with_label("Theme:");
    let mut theme_choice = MenuChoice::default();
    theme_choice.add_choice(
        "Hakurei Serenity|Spring of the Valley|Eternal Night|\
         Grave of the Fireflies|The Sea of Ponyo|Forest of Magic|\
         Forest Spirit|Howl's Golden Autumn|Hakugyokurou|Kiki's Afternoon"
    );
    theme_choice.set_value(0);

    let url_label = fltk::frame::Frame::default().with_label("API Endpoint URL:");
    let mut url_input = Input::default();
    url_input.set_value(&data.api_url);

    let key_label = fltk::frame::Frame::default().with_label("API Key:");
    let mut key_input = SecretInput::default();
    key_input.set_value(&data.api_key);

    let model_label = fltk::frame::Frame::default().with_label("Model:");
    let mut model_input = Input::default();
    model_input.set_value(&data.model);

    let mut clear_btn = Button::default().with_label("Clear Chat History");
    
    sidebar.end();
    outer_flex.fixed(&sidebar, 260);

    // --- RIGHT PANEL: CHAT AREA ---
    let mut chat_flex = Flex::default()
        .with_size(680, 650)
        .with_type(FlexType::Column);
    chat_flex.set_spacing(8);

    let mut chat_display = TextDisplay::default();
    let chat_buffer = TextBuffer::default();
    chat_display.set_buffer(chat_buffer.clone());
    chat_display.set_text_font(Font::Courier);
    chat_display.set_text_size(14);

    // Markdown & display formatter helper
    let mut update_display = {
        let mut buf = chat_buffer.clone();
        move |msgs: &[Message]| {
            let mut text = String::new();
            for msg in msgs {
                let content_str = match &msg.content {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => {
                        arr.iter()
                            .filter_map(|item| {
                                if item.get("type")? == "text" {
                                    Some(item.get("text")?.as_str()?.to_string())
                                } else if item.get("type")? == "image_url" {
                                    Some("[Attached Image]".to_string())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                    _ => "".to_string(),
                };

                if msg.role == "system" && !content_str.starts_with("⚠️") {
                    continue;
                }

                let prefix = match msg.role.as_str() {
                    "user" => "🧑 You:\n",
                    "assistant" => "🍮 Custard:\n",
                    _ => "",
                };

                let formatted_content = content_str
                    .replace("### ", "--- \n### ")
                    .replace("**", "");

                text.push_str(&format!("{}{}\n\n", prefix, formatted_content));
            }
            buf.set_text(&text);
        }
    };

    update_display(&data.messages);

    // Bottom input row (Attach Image + Text input + Send button)
    let mut input_row = Flex::default()
        .with_size(680, 40)
        .with_type(FlexType::Row);
    input_row.set_spacing(6);

    let mut attach_btn = Button::default().with_label("🖼️");
    input_row.fixed(&attach_btn, 40);

    let msg_input = Input::default();
    let mut send_btn = Button::default().with_label("Send");
    input_row.fixed(&send_btn, 80);
    input_row.end();

    chat_flex.end();
    outer_flex.end();
    wind.end();
    wind.make_resizable(true);
    wind.show();

    // --- THEME SWITCHING LOGIC ---
    let mut wind_clone = wind.clone();
    let mut outer_clone = outer_flex.clone();
    let mut sidebar_clone = sidebar.clone();
    let mut chat_clone = chat_flex.clone();
    let mut input_row_clone = input_row.clone();

    let mut display_clone = chat_display.clone();

    let mut url_clone = url_input.clone();
    let mut key_clone = key_input.clone();
    let mut model_clone = model_input.clone();
    let mut choice_clone = theme_choice.clone();

    let mut send_clone_btn = send_btn.clone();
    let mut attach_clone_btn = attach_btn.clone();
    let mut clear_clone_btn = clear_btn.clone();

    let mut title_clone = sidebar_title.clone();
    let mut theme_label_clone = theme_label.clone();
    let mut url_label_clone = url_label.clone();
    let mut key_label_clone = key_label.clone();
    let mut model_label_clone = model_label.clone();

    theme_choice.set_callback(move |choice| {
        let p = palette_for(choice.value());

        wind_clone.set_color(p.bg);
        outer_clone.set_color(p.bg);
        sidebar_clone.set_color(p.panel);
        chat_clone.set_color(p.bg);
        input_row_clone.set_color(p.panel);

        display_clone.set_color(p.panel);
        display_clone.set_text_color(p.text);
        let _ = p.border;

        url_clone.set_color(p.input);
        url_clone.set_text_color(p.text);

        key_clone.set_color(p.input);
        key_clone.set_text_color(p.text);

        model_clone.set_color(p.input);
        model_clone.set_text_color(p.text);

        choice_clone.set_color(p.input);
        choice_clone.set_text_color(p.text);

        send_clone_btn.set_color(p.button);
        send_clone_btn.set_label_color(p.button_text);

        attach_clone_btn.set_color(p.button);
        attach_clone_btn.set_label_color(p.button_text);

        clear_clone_btn.set_color(p.button);
        clear_clone_btn.set_label_color(p.button_text);

        title_clone.set_color(p.panel);
        title_clone.set_label_color(p.accent);

        theme_label_clone.set_color(p.panel);
        theme_label_clone.set_label_color(p.muted);

        url_label_clone.set_color(p.panel);
        url_label_clone.set_label_color(p.muted);

        key_label_clone.set_color(p.panel);
        key_label_clone.set_label_color(p.muted);

        model_label_clone.set_color(p.panel);
        model_label_clone.set_label_color(p.muted);

        wind_clone.redraw();
    });

    // Initialize with default theme (Hakurei Serenity)
    theme_choice.do_callback();

    let (s, r) = app::channel::<MessageAction>();

    // Inputs persistence
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

    let mut data_clone4 = data.clone();
    let mut disp_ref = update_display.clone();
    clear_btn.set_callback(move |_| {
        data_clone4.messages.truncate(1);
        save_to_disk(&data_clone4);
        disp_ref(&data_clone4.messages);
    });

    // Image attachment state holder
    let attached_image_path: std::rc::Rc<std::cell::RefCell<Option<String>>> = 
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let img_path_clone = attached_image_path.clone();
    attach_btn.set_callback(move |_| {
        if let Some(path) = dialog::file_chooser("Choose an image", "*.{png,jpg,jpeg,webp}", "", false) {
            *img_path_clone.borrow_mut() = Some(path);
            dialog::message_default("Image attached successfully!");
        }
    });

    // Send message action
    let send_action = {
        let mut msg_input = msg_input.clone();
        let url_in = url_input.clone();
        let key_in = key_input.clone();
        let model_in = model_input.clone();
        let mut data = data.clone();
        let mut disp_ref2 = update_display.clone();
        let sender = s;
        let img_path_ref = attached_image_path.clone();

        move || {
            let text = msg_input.value().trim().to_string();
            let mut current_img = img_path_ref.borrow_mut();

            if text.is_empty() && current_img.is_none() {
                return;
            }

            data.api_url = url_in.value();
            data.api_key = key_in.value();
            data.model = model_in.value();

            let message_content = if let Some(path) = current_img.take() {
                if let Ok(bytes) = fs::read(&path) {
                    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
                    let ext = path.split('.').last().unwrap_or("png");
                    let data_url = format!("data:image/{};base64,{}", ext, b64);

                    json!([
                        { "type": "text", "text": if text.is_empty() { "What is in this image?".to_string() } else { text } },
                        { "type": "image_url", "image_url": { "url": data_url } }
                    ])
                } else {
                    json!(text)
                }
            } else {
                json!(text)
            };

            data.messages.push(Message {
                role: "user".to_owned(),
                content: message_content,
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

    let mut send_clone = send_action;
    send_btn.set_callback(move |_| {
        send_clone();
    });

    let mut active_data = data;
    while app.wait() {
        if let Some(action) = r.recv() {
            match action {
                MessageAction::ApiResult(result) => {
                    match result {
                        Ok(reply) => {
                            active_data.messages.push(Message {
                                role: "assistant".to_owned(),
                                content: json!(reply),
                            });
                        }
                        Err(err) => {
                            active_data.messages.push(Message {
                                role: "system".to_owned(),
                                content: json!(format!("⚠️ {}", err)),
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
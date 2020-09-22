use csml_interpreter::{
    data::{csml_bot::CsmlBot, csml_flow::CsmlFlow, Client},
    load_components,
};
use csml_manager::{data::CsmlRequest, start_conversation};
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdin;

fn get_flow(name: &str) -> Result<String, ::std::io::Error> {
    let file_path = format!("CSML/{}.csml", name);

    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn init_request(string: &str) -> CsmlRequest {
    CsmlRequest {
        request_id: "tmp".to_owned(),
        client: Client {
            user_id: "alexis".to_owned(),
            bot_id: "42".to_owned(),
            channel_id: "some-channel-id".to_owned(),
        },
        callback_url: Some("http://httpbin.org/post".to_owned()),
        payload: json!({
            "content_type": "text",
            "content": { "text": string},
        }),
        metadata: json!({"some": "custom-value"}),
    }
}

fn init_bot() -> CsmlBot {
    CsmlBot {
        id: "botid".to_owned(),
        name: "plop".to_owned(),
        fn_endpoint: Some("endpoint".to_owned()),
        flows: vec![
            CsmlFlow {
                id: "flowid".to_owned(),
                name: "flow".to_owned(),
                content: get_flow("flow").expect("error in reading flow"),
                commands: vec!["/plop".to_owned()],
            },
            CsmlFlow {
                id: "2".to_owned(),
                name: "flow2".to_owned(),
                content: get_flow("flow2").expect("error in reading flow"),
                commands: vec!["/random".to_owned()],
            },
        ],
        native_components: Some(load_components().unwrap()),
        custom_components: None,
        default_flow: "flowid".to_owned(),
    }
}

fn main() {
    let mut line: String = String::new();
    // let mut context = init_context();
    loop {
        let data = init_bot();
        stdin()
            .read_line(&mut line)
            .ok()
            .expect("Failed to read line :)");
        if line.trim().is_empty() {
            continue;
        }
        let input = line.trim().to_owned();
        if input == "exit" {
            break;
        }
        match start_conversation(init_request(&input), data) {
            Ok(obj) => {
                if obj["conversation_end"].as_bool().unwrap() {
                    break;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }
        line.clear();
    }
}

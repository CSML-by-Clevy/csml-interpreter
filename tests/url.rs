mod support;

use csmlinterpreter::{interpret};
use csmlinterpreter::interpreter::{json_to_rust::*, message::{MessageData}};
use csmlinterpreter::parser::Parser;
use serde_json::Value;
use multimap::MultiMap;

use support::tools::{gen_context, message_to_jsonvalue, read_file};

fn format_message(event: Option<Event>, step: &str) -> MessageData {
    let text = read_file("CSML/built-in/url.csml".to_owned()).unwrap();
    let flow = Parser::parse_flow(text.as_bytes()).unwrap();

    let memory = gen_context(MultiMap::new(), MultiMap::new(), MultiMap::new(), 0, false);

    interpret(&flow, step, &memory, &event)
}

#[test]
fn ok_url() {
    let data = r#"{"messages":[ {"content":{ "url": {"url": "test", "text": "test", "title": "test"} },"content_type":"url"} ],"next_flow":null,"memories":[],"next_step":"end"}"#;
    let msg = format_message(None, "start");

    let v1: Value = message_to_jsonvalue(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2)
}

#[test]
fn ok_url_step1() {
    let data = r#"{"messages":[ {"content":{ "url": {"url": "test", "text": "test", "title": "test"} },"content_type":"url"} ],"next_flow":null,"memories":[],"next_step":"end"}"#;
    let msg = format_message(None, "url1");

    let v1: Value = message_to_jsonvalue(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2)
}

#[test]
fn ok_url_step2() {
    let data = r#"{"messages":[ {"content":{ "url": {"url": "test", "text": "plop", "title": "test"} },"content_type":"url"} ],"next_flow":null,"memories":[],"next_step":"end"}"#;
    let msg = format_message(None, "url2");

    let v1: Value = message_to_jsonvalue(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2)
}

#[test]
fn ok_url_step3() {
    let data = r#"{"messages":[ {"content":{ "url": {"url": "test", "text": "plop", "title": "rand"} },"content_type":"url"} ],"next_flow":null,"memories":[],"next_step":"end"}"#;
    let msg = format_message(None, "url3");

    let v1: Value = message_to_jsonvalue(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2)
}
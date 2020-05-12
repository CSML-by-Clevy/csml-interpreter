use crate::data::{primitive::{PrimitiveType, PrimitiveString, PrimitiveObject}};
use crate::data::{ast::Interval, tokens::*, ApiInfo, Client, Data, Literal};
use crate::interpreter::{builtins::{http::http_request, tools::*}};
use crate::error_format::*;

use std::{collections::HashMap, env};

fn format_body(
    args: &HashMap<String, Literal>,
    interval: Interval,
    client: Client,
) -> Result<HashMap<String, Literal>, ErrorInfo> {
    let mut map: HashMap<String, Literal> = HashMap::new();

    match (args.get("fn_id"), args.get(DEFAULT)) {
        (Some(literal), ..) | (.., Some(literal))
            if literal.primitive.get_type() == PrimitiveType::PrimitiveString =>
        {
            let fn_id = Literal::get_value::<String>(
                &literal.primitive,
                literal.interval,
                ERROR_FN_ID.to_owned(),
            )?;

            map.insert(
                "function_id".to_owned(),
                PrimitiveString::get_literal(&fn_id, interval),
            );
        }
        _ => return Err(gen_error_info(interval, ERROR_FN_ID.to_owned())),
    };

    let sub_map = create_submap(&["fn_id", DEFAULT], &args)?;
    let client = client_to_json(&client, interval);

    map.insert("data".to_owned(), PrimitiveObject::get_literal(&sub_map, interval));
    map.insert("client".to_owned(), PrimitiveObject::get_literal(&client, interval));

    let mut body: HashMap<String, Literal> = HashMap::new();
    body.insert("body".to_owned(), PrimitiveObject::get_literal(&map, interval));
    Ok(body)
}

fn format_headers(interval: Interval) -> HashMap<String, Literal> {
    let mut header = HashMap::new();
    header.insert(
        "content-type".to_owned(),
        PrimitiveString::get_literal("application/json", interval),
    );
    header.insert(
        "accept".to_owned(),
        PrimitiveString::get_literal("application/json,text/*", interval),
    );
    
    match env::var("FN_X_API_KEY") {
        Ok(value) => header.insert(
            "X-Api-Key".to_owned(),
            PrimitiveString::get_literal(&value, interval)
        ),
        Err(_e) => header.insert(
            "X-Api-Key".to_owned(),
            PrimitiveString::get_literal("PoePoe", interval)
        )
    };

    header
}

pub fn api(
    args: HashMap<String, Literal>,
    interval: Interval,
    data: &mut Data,
) -> Result<Literal, ErrorInfo> {
    let (client, url) = match &data.context.api_info {
        Some(ApiInfo {
            client,
            fn_endpoint,
        }) => (client.to_owned(), fn_endpoint.to_owned()),
        None => return Err(gen_error_info(interval, ERROR_FN_ENDPOINT.to_owned())),
    };

    let mut http: HashMap<String, Literal> = HashMap::new();
    let header = format_headers(interval);
    let body = format_body(&args, interval, client)?;

    http.insert("url".to_owned(),  PrimitiveString::get_literal(&url, interval));

    let lit_header = PrimitiveObject::get_literal(&header, interval);
    http.insert("header".to_owned(), lit_header);
    let lit_query = PrimitiveObject::get_literal(&HashMap::default(), interval);
    http.insert("query".to_owned(), lit_query);
    let lit_body = PrimitiveObject::get_literal(&body, interval);
    http.insert("body".to_owned(), lit_body);

    http_request(&http, ureq::post, interval)
}
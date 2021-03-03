use crate::{Client, Context, db_connectors, encrypt::{decrypt_data, encrypt_data}};
use csml_interpreter::data::{CsmlBot, CsmlFlow, Message};
use curl::easy::Easy;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEBUG: &str = "DEBUG";
pub const DISABLE_SSL_VERIFY: &str = "DISABLE_SSL_VERIFY";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunRequest {
    pub bot: Option<CsmlBot>,
    pub bot_id: Option<String>,
    pub version_id: Option<String>,
    pub fn_endpoint: Option<String>,
    pub event: CsmlRequest,
}

impl RunRequest {
    pub fn get_bot_opt(&self) -> Result<BotOpt, EngineError> {
        match self.clone() {
            RunRequest {
                bot: Some(csml_bot),
                ..
            } => Ok(BotOpt::CsmlBot(csml_bot)),
            RunRequest {
                version_id: Some(version_id),
                bot_id: Some(bot_id),
                fn_endpoint,
                ..
            } => Ok(BotOpt::Id {
                version_id,
                bot_id,
                fn_endpoint,
            }),
            RunRequest {
                bot_id: Some(bot_id),
                fn_endpoint,
                ..
            } => Ok(BotOpt::BotId {
                bot_id,
                fn_endpoint,
            }),
            _ => Err(EngineError::Format("Invalid bot_opt format".to_owned())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BotOpt {
    #[serde(rename = "bot")]
    CsmlBot(CsmlBot),
    #[serde(rename = "version_id")]
    Id {
        version_id: String,
        bot_id: String,
        fn_endpoint: Option<String>,
    },
    #[serde(rename = "bot_id")]
    BotId {
        bot_id: String,
        fn_endpoint: Option<String>,
    },
}

impl BotOpt {
    pub fn search_bot(&self, db: &mut Database) -> CsmlBot {
        match self {
            BotOpt::CsmlBot(csml_bot) => csml_bot.to_owned(),
            BotOpt::BotId {
                bot_id,
                fn_endpoint,
            } => {
                let mut bot_version = db_connectors::bot::get_last_bot_version(&bot_id, db)
                    .unwrap()
                    .unwrap();
                bot_version.bot.fn_endpoint = fn_endpoint.to_owned();
                bot_version.bot
            }
            BotOpt::Id {
                version_id,
                bot_id,
                fn_endpoint,
            } => {
                let mut bot_version =
                    db_connectors::bot::get_by_version_id(&version_id, &bot_id, db)
                        .unwrap()
                        .unwrap();
                bot_version.bot.fn_endpoint = fn_endpoint.to_owned();
                bot_version.bot
            }
        }
    }
}

fn encrypt_env(mut env: serde_json::Value) -> Result<String, EngineError> {
    match env.as_object_mut() {
        Some(obj) => {
            for (k, v) in obj.iter_mut() {
                if !v.is_string() {
                    return Err(EngineError::Format("env values need to be of type string".to_owned()));
                }
                let encrypted_value = match encrypt_data(&v) {
                    Ok(value) => value,
                    Err(_) => return Err(EngineError::Format(format!("Invalid [{}] value in env", k)))
                };
                *v = serde_json::json!(encrypted_value);
            };

            Ok(serde_json::json!(obj).to_string())
        }
        None => Err(EngineError::Format("Invalid env format".to_owned()))
    }
}

fn decrypt_env(encrypted_env: String) -> Result<serde_json::Value, EngineError> {
    let mut env = match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&encrypted_env) {
        Ok(value) => value,
        Err(_e) => return Err(EngineError::Format("corrupted encrypted env value".to_owned())),
    };

    for (k, v) in env.iter_mut() {
        if !v.is_string() {
            return Err(EngineError::Format("env values need to be of type string".to_owned()));
        }
        match v.as_str() {
            Some(value) => {
                *v = match decrypt_data(value.to_owned()) {
                    Ok(value) => value,
                    Err(_) => return Err(EngineError::Format(format!("decryption fail for env value [{}]", k)))
                };
            }
            None => return Err(EngineError::Format(format!("Invalid [{}] value in env", k)))
        }
    };

    Ok(serde_json::json!(env))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializeCsmlBot {
    pub id: String,
    pub name: String,
    pub flows: Vec<CsmlFlow>,
    pub native_components: Option<String>, // serde_json::Map<String, serde_json::Value>
    pub custom_components: Option<String>, // serde_json::Value
    pub default_flow: String,
    pub env: Option<String>,
}

impl SerializeCsmlBot {
    pub fn to_bot(&self) -> Result<CsmlBot, EngineError> {
        Ok(CsmlBot {
            id: self.id.to_owned(),
            name: self.name.to_owned(),
            fn_endpoint: None,
            flows: self.flows.to_owned(),
            native_components: {
                match self.native_components.to_owned() {
                    Some(value) => match serde_json::from_str(&value) {
                        Ok(serde_json::Value::Object(map)) => Some(map),
                        _ => unreachable!(),
                    },
                    None => None,
                }
            },
            custom_components: {
                match self.custom_components.to_owned() {
                    Some(value) => match serde_json::from_str(&value) {
                        Ok(value) => Some(value),
                        Err(_e) => unreachable!(),
                    },
                    None => None,
                }
            },
            default_flow: self.default_flow.to_owned(),
            bot_ast: None,
            env: match self.custom_components.to_owned() {
                Some(value) => Some(decrypt_env(value)?),
                None => None,
            }
        })
    }

    pub fn serialize_bot(bot: &CsmlBot) -> Result<SerializeCsmlBot, EngineError> {
        Ok(SerializeCsmlBot {
            id: bot.id.to_owned(),
            name: bot.name.to_owned(),
            flows: bot.flows.to_owned(),
            native_components: {
                match bot.native_components.to_owned() {
                    Some(value) => Some(serde_json::Value::Object(value).to_string()),
                    None => None,
                }
            },
            custom_components: {
                match bot.custom_components.to_owned() {
                    Some(value) => Some(value.to_string()),
                    None => None,
                }
            },
            default_flow: bot.default_flow.to_owned(),
            env: match &bot.env {
                Some(value) => Some(encrypt_env(value.to_owned())?),
                None => None,
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamoBot {
    pub id: String,
    pub name: String,
    pub custom_components: Option<String>,
    pub default_flow: String,
    pub env: Option<String>
}

impl DynamoBot {
    pub fn to_bot(&self, flows: Vec<CsmlFlow>) -> Result<CsmlBot, EngineError> {
        Ok(CsmlBot {
            id: self.id.to_owned(),
            name: self.name.to_owned(),
            fn_endpoint: None,
            flows,
            native_components: None,
            custom_components: {
                match self.custom_components.to_owned() {
                    Some(value) => match serde_json::from_str(&value) {
                        Ok(value) => Some(value),
                        Err(_e) => unreachable!(),
                    },
                    None => None,
                }
            },
            default_flow: self.default_flow.to_owned(),
            bot_ast: None,
            env: match self.env.to_owned() {
                Some(value) => Some(decrypt_env(value.to_owned())?),
                None => None,
            },
        })
    }

    pub fn serialize_bot(csml_bot: &CsmlBot) -> Result<DynamoBot, EngineError> {
        Ok(DynamoBot {
            id: csml_bot.id.to_owned(),
            name: csml_bot.name.to_owned(),
            custom_components: match csml_bot.custom_components.to_owned() {
                Some(value) => Some(value.to_string()),
                None => None,
            },
            default_flow: csml_bot.default_flow.to_owned(),
            env: match &csml_bot.env {
                Some(value) => Some(encrypt_env(value.to_owned())?),
                None => None,
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsmlRequest {
    pub request_id: String,
    pub client: Client,
    pub callback_url: Option<String>,
    pub payload: serde_json::Value,
    pub metadata: serde_json::Value,
}

pub enum Database {
    #[cfg(feature = "mongo")]
    Mongo(mongodb::Database),
    #[cfg(feature = "dynamo")]
    Dynamodb(DynamoDbClient),
    None,
}

/**
 * Dynamodb runs in async by default and returns futures, that need to be awaited on.
 * The proper way to do it is by using tokio's runtime::block_on(). It is however quite costly
 * to setup, so let's just do it once in the base DynamoDbStruct here.
 */
#[cfg(feature = "dynamo")]
pub struct DynamoDbClient {
    pub client: rusoto_dynamodb::DynamoDbClient,
    pub s3_client: rusoto_s3::S3Client,
    pub runtime: tokio::runtime::Runtime,
}

#[cfg(feature = "dynamo")]
impl DynamoDbClient {
    pub fn new(dynamo_region: rusoto_core::Region, s3_region: rusoto_core::Region) -> Self {
        Self {
            client: rusoto_dynamodb::DynamoDbClient::new(dynamo_region),
            s3_client: rusoto_s3::S3Client::new(s3_region),
            runtime: { tokio::runtime::Runtime::new().unwrap() },
        }
    }
}

pub struct ConversationInfo {
    pub request_id: String,
    pub curl: Option<Easy>,
    pub conversation_id: String,
    pub interaction_id: String,
    pub client: Client,
    pub context: Context,
    pub metadata: Value,
    pub messages: Vec<Message>,
    pub db: Database,
}

#[derive(Debug)]
pub enum Next {
    Flow(String),
    Step(String),
    Hold, //(i32)
    End,
    Error,
}

#[derive(Debug)]
pub enum EngineError {
    Serde(serde_json::Error),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Manager(String),
    Format(String),
    Interpreter(String),
    Time(std::time::SystemTimeError),
    Openssl(openssl::error::ErrorStack),
    Base64(base64::DecodeError),

    #[cfg(any(feature = "mongo"))]
    BsonDecoder(bson::DecoderError),
    #[cfg(any(feature = "mongo"))]
    BsonEncoder(bson::EncoderError),
    #[cfg(any(feature = "mongo"))]
    MongoDB(mongodb::error::Error),

    #[cfg(any(feature = "dynamo"))]
    Rusoto(String),
    #[cfg(any(feature = "dynamo"))]
    SerdeDynamodb(serde_dynamodb::Error),
    #[cfg(any(feature = "dynamo"))]
    S3ErrorCode(u16),
}

impl From<serde_json::Error> for EngineError {
    fn from(e: serde_json::Error) -> Self {
        EngineError::Serde(e)
    }
}

impl From<std::io::Error> for EngineError {
    fn from(e: std::io::Error) -> Self {
        EngineError::Io(e)
    }
}

impl From<std::str::Utf8Error> for EngineError {
    fn from(e: std::str::Utf8Error) -> Self {
        EngineError::Utf8(e)
    }
}

impl From<std::time::SystemTimeError> for EngineError {
    fn from(e: std::time::SystemTimeError) -> Self {
        EngineError::Time(e)
    }
}

impl From<openssl::error::ErrorStack> for EngineError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        EngineError::Openssl(e)
    }
}

impl From<base64::DecodeError> for EngineError {
    fn from(e: base64::DecodeError) -> Self {
        EngineError::Base64(e)
    }
}

#[cfg(any(feature = "mongo"))]
impl From<bson::EncoderError> for EngineError {
    fn from(e: bson::EncoderError) -> Self {
        EngineError::BsonEncoder(e)
    }
}

#[cfg(any(feature = "mongo"))]
impl From<bson::DecoderError> for EngineError {
    fn from(e: bson::DecoderError) -> Self {
        EngineError::BsonDecoder(e)
    }
}

#[cfg(any(feature = "mongo"))]
impl From<mongodb::error::Error> for EngineError {
    fn from(e: mongodb::error::Error) -> Self {
        EngineError::MongoDB(e)
    }
}

#[cfg(any(feature = "dynamo"))]
impl<E: std::error::Error + 'static> From<rusoto_core::RusotoError<E>> for EngineError {
    fn from(e: rusoto_core::RusotoError<E>) -> Self {
        EngineError::Rusoto(e.to_string())
    }
}

#[cfg(any(feature = "dynamo"))]
impl From<serde_dynamodb::Error> for EngineError {
    fn from(e: serde_dynamodb::Error) -> Self {
        EngineError::SerdeDynamodb(e)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_env_encryption() {
        let env = serde_json::json!({
            "key": "value",
            "test": "toto"
        });

        let encrypted_env = encrypt_env(env.clone()).unwrap();

        let decrypted_env = decrypt_env(encrypted_env).unwrap();

        assert_eq!(env, decrypted_env);
    }
}

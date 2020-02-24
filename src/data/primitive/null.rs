use crate::data::primitive::object::PrimitiveObject;
use crate::data::primitive::string::PrimitiveString;
use crate::data::primitive::tools::check_usage;
use crate::data::primitive::Right;
use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::{
    ast::Interval, memories::MemoryType, message::Message, tokens::NULL_UPPERCASE, Literal,
};
use crate::error_format::ErrorInfo;
use lazy_static::*;
use std::cmp::Ordering;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    null: &mut PrimitiveNull,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

lazy_static! {
    static ref FUNCTIONS: HashMap<&'static str, (PrimitiveMethod, Right)> = {
        let mut map = HashMap::new();

        // type_of() -> Primitive<String>
        map.insert("type_of", (type_of as PrimitiveMethod, Right::Read));

        // to_string() -> Primitive<String>
        map.insert("to_string", (to_string as PrimitiveMethod, Right::Read));

        map
    };
}

#[derive(PartialEq, Debug, Clone)]
pub struct PrimitiveNull {}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn type_of(
    _null: &mut PrimitiveNull,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "type_of()", interval)?;

    Ok(PrimitiveString::get_literal("string", "null", interval))
}

fn to_string(
    null: &mut PrimitiveNull,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "to_string()", interval)?;

    Ok(PrimitiveString::get_literal(
        "string",
        &null.to_string(),
        interval,
    ))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl Default for PrimitiveNull {
    fn default() -> Self {
        Self {}
    }
}

impl PrimitiveNull {
    pub fn get_literal(content_type: &str, interval: Interval) -> Literal {
        let primitive = Box::new(PrimitiveNull::default());

        Literal {
            content_type: content_type.to_owned(),
            primitive,
            interval,
        }
    }
}

impl Primitive for PrimitiveNull {
    fn do_exec(
        &mut self,
        name: &str,
        args: &[Literal],
        interval: Interval,
        _mem_type: &MemoryType,
    ) -> Result<(Literal, Right), ErrorInfo> {
        if let Some((f, right)) = FUNCTIONS.get(name) {
            let res = f(self, args, interval)?;

            return Ok((res, *right));
        }

        Err(ErrorInfo {
            message: format!("unknown method '{}' for type Null", name),
            interval,
        })
    }

    fn is_eq(&self, other: &dyn Primitive) -> bool {
        if let Some(_other) = other.as_any().downcast_ref::<Self>() {
            return true;
        }

        false
    }

    fn is_cmp(&self, _other: &dyn Primitive) -> Option<Ordering> {
        None
    }

    fn do_add(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn do_sub(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn do_div(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn do_mul(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn do_rem(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn do_bitand(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn do_bitor(&self, _other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        Ok(Box::new(PrimitiveNull::default()))
    }

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveNull
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn to_string(&self) -> String {
        "Null".to_owned()
    }

    fn as_bool(&self) -> bool {
        false
    }

    fn get_value(&self) -> &dyn std::any::Any {
        &NULL_UPPERCASE
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn to_msg(&self, _content_type: String) -> Message {
        let mut hashmap: HashMap<String, Literal> = HashMap::new();

        hashmap.insert(
            "text".to_owned(),
            Literal {
                content_type: "text".to_owned(),
                primitive: Box::new(PrimitiveNull::default()),
                interval: Interval { column: 0, line: 0 },
            },
        );

        let result =
            PrimitiveObject::get_literal("text", &hashmap, Interval { column: 0, line: 0 });

        Message {
            content_type: result.content_type,
            content: result.primitive.to_json(),
        }
    }
}

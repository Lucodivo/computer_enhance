use std::io::{Result, Error, ErrorKind::InvalidData};

#[allow(unused_imports)]
use profiler::*;

// Note: This does not follow spec exactly.
// Known things that don't match spec:
//  - Root must be an object/array
//  - Anything following the root object/array is ignored
//  - Commas are ignored and treated the same as whitespace
//  - (Spec is uncertain on this but...) Keys in objects are not unique.
//      - HashMap/FxHashMap were slower than linear searching in my limited usecase.
//  - Strings are returned as just the bytes between two non-escaped quotes. Escape slashes and their following character are not removed.

pub enum JsonValue<'a> {
    Object{ start_index: usize, end_index: usize },
    Array{ start_index: usize, end_index: usize },
    String(&'a[u8]),
    Number(f64),
    Boolean(bool),
    Null
}

enum JsonToken<'a> {
    ObjectStart, ObjectTerminate,
    ArrayStart, ArrayTerminate,
    String(&'a[u8]),
    Number(f64),
    Boolean(bool),
    Null
}

pub struct Json<'a> {
    bytes: &'a [u8],
    root: JsonValue<'a>,
    object_members: Vec<(&'a[u8], JsonValue<'a>)>, // Note: "Keys" are not necessarily unique within a json object :(
    array_elements: Vec<JsonValue<'a>>
}

pub fn parse_json_bytes<'a>(json_bytes: &'a [u8]) -> Result<Json<'a>> {
    time_function!();

    let mut json_data: Json<'a> = Json {
        bytes: json_bytes,
        root: JsonValue::Null,
        object_members: Vec::<(&[u8], JsonValue)>::new(),
        array_elements: Vec::<JsonValue>::new()
    };
    let mut processing_index = 0;
    json_data.root = match parse_token(json_bytes, &mut processing_index) {
        Ok(JsonToken::ObjectStart) => {
            match parse_json_object(&mut json_data, &mut processing_index) {
                Ok(root) => { root },
                Err(e) => return Err(e)
            }
        },
        Ok(JsonToken::ArrayStart) => {
            match parse_json_array(&mut json_data, &mut processing_index) {
                Ok(root) => { root },
                Err(e) => return Err(e)
            }
        },
        Ok(_) => return Err(Error::new(InvalidData, "ERROR: Json must have an object or array as the root. Nothing may exist but whitespace after the root.")),
        Err(e) => return Err(e)
    };

    return Ok(json_data);
}

fn parse_json_object<'a>(json: &mut Json<'a>, parsing_index: &mut usize) -> Result<JsonValue<'a>> {
    let start = json.object_members.len();
    loop {
        match parse_token(json.bytes, parsing_index)? {
            JsonToken::String(key_str) => {
                match parse_token(json.bytes, parsing_index)? {
                    JsonToken::ObjectStart => {
                        let object_index = json.object_members.len();
                        json.object_members.push((key_str, JsonValue::Null)); // Easier navigation if the parent's index is it's children.
                        json.object_members[object_index].1 = parse_json_object(json, parsing_index)?;
                    },
                    JsonToken::ArrayStart => {
                        let array_index = json.object_members.len();
                        json.object_members.push((key_str, JsonValue::Null)); // Easier navigation if the parent's index is it's children.
                        json.object_members[array_index].1 = parse_json_array(json, parsing_index)?;
                    },
                    JsonToken::String(s) => json.object_members.push((key_str, JsonValue::String(s))),
                    JsonToken::Number(n) => json.object_members.push((key_str, JsonValue::Number(n))),
                    JsonToken::Boolean(b) => json.object_members.push((key_str, JsonValue::Boolean(b))),
                    JsonToken::Null => json.object_members.push((key_str, JsonValue::Null)),
                    JsonToken::ObjectTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected object member value but found end of object.")); }
                    JsonToken::ArrayTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected object member value but found end of array.")); }
                };
            },
            JsonToken::ObjectTerminate => { return Ok(JsonValue::Object{ start_index: start, end_index: json.object_members.len() }); },
            _ => { return Err(Error::new(InvalidData, "ERROR: Expected a string for a key in a json object.")) }
        }
    }
}

fn parse_json_array<'a>(json: &mut Json<'a>, parsing_index: &mut usize) -> Result<JsonValue<'a>> {
    let start = json.array_elements.len();
    loop {
        let token = parse_token(json.bytes, parsing_index)?;
        match token {
            JsonToken::ObjectStart => {
                let object_index = json.array_elements.len();
                json.array_elements.push(JsonValue::Null); // Easier navigation if the parent's index is it's children.
                let json_object = parse_json_object(json, parsing_index)?;
                json.array_elements[object_index] = json_object;
            },
            JsonToken::ArrayStart => {
                let array_index = json.array_elements.len();
                json.array_elements.push(JsonValue::Null); // Easier navigation if the parent's index is it's children.
                let json_array = parse_json_array(json, parsing_index)?;
                json.array_elements[array_index] = json_array;
            },
            JsonToken::String(s) => json.array_elements.push(JsonValue::String(s)),
            JsonToken::Number(n) => json.array_elements.push(JsonValue::Number(n)),
            JsonToken::Boolean(b) => json.array_elements.push(JsonValue::Boolean(b)),
            JsonToken::Null => json.array_elements.push(JsonValue::Null),
            JsonToken::ArrayTerminate => { return Ok(JsonValue::Array{ start_index: start, end_index: json.array_elements.len() }); },
            JsonToken::ObjectTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected array element but found end of an object.")); }
        }
    }
}

fn parse_token<'a>(json_bytes: &'a [u8], parsing_index: &mut usize) -> Result<JsonToken<'a>> {

    while *parsing_index < json_bytes.len() {

        let check_slice = |s: &[u8], i: &mut usize| -> bool {
            let start_i = *i;
            *i += s.len();
            if json_bytes.len() < *i { return false; }
            return json_bytes[start_i..*i].eq(s)
        };

        let mut byte = json_bytes[*parsing_index];
        *parsing_index += 1; // index first, as it is expected to advance before returning from function
        match byte {
            b'{' => { return Ok(JsonToken::ObjectStart) },
            b'}' => { return Ok(JsonToken::ObjectTerminate) },
            b'[' => { return Ok(JsonToken::ArrayStart) },
            b']' => { return Ok(JsonToken::ArrayTerminate) },
            b'"' => {
                let start_str_index = *parsing_index;
                while *parsing_index < json_bytes.len() {
                    match json_bytes[*parsing_index] {
                        b'\\' => { *parsing_index += 2; }
                        b'"' => {
                            let end_quote_index = *parsing_index;
                            *parsing_index += 1;
                            return Ok(JsonToken::String(&json_bytes[start_str_index..end_quote_index]));
                        },
                        _ => { *parsing_index += 1; }
                    }
                }
            },
            b'0'..=b'9'|b'-'|b'+' => {

                // pull sign
                let sign: f64;
                if byte == b'-' {
                    sign = -1.0;
                    byte = json_bytes[*parsing_index];
                    *parsing_index += 1;
                } else {
                    sign = 1.0;
                    if byte == b'+' { 
                        byte = json_bytes[*parsing_index];
                        *parsing_index += 1; 
                    }
                }

                // pull integer
                let mut integer: f64 = (byte - b'0') as f64;
                while *parsing_index < json_bytes.len() {
                    byte = json_bytes[*parsing_index];

                    match byte {
                        b'0'..=b'9' => {
                            integer = (integer * 10.0) + (byte - b'0') as f64;
                            *parsing_index += 1;
                        },
                        _ => {break;}
                    }
                }

                // pull fraction
                let mut fract: f64 = 0.0;
                if byte == b'.' {
                    *parsing_index += 1;
                    let mut fract_mul: f64 = 1.0 / 10.0;

                    while *parsing_index < json_bytes.len() {
                        byte = json_bytes[*parsing_index];
    
                        match byte {
                            b'0'..=b'9' => {
                                fract += fract_mul * (byte - b'0') as f64;
                                fract_mul *= 1.0 / 10.0;
                                *parsing_index += 1; 
                            },
                            _ => {break;}
                        }
                    }
                }

                // pull explicit exponent
                let mut exp: i32 = 0;
                let mut exp_sign: i32 = 1;
                if byte == b'e' || byte == b'E' {
                    *parsing_index += 1;

                    if json_bytes[*parsing_index] == b'-' {
                        exp_sign = -1;
                        *parsing_index += 1;
                    } else if byte == b'+' { 
                        *parsing_index += 1; 
                    }

                    while *parsing_index < json_bytes.len() {
                        byte = json_bytes[*parsing_index];
    
                        match byte {
                            b'0'..=b'9' => {
                                exp *= 10;
                                exp += (byte - b'0') as i32;
                                *parsing_index += 1; 
                            },
                            _ => { break; }
                        }
                    }
                }
                
                let exp_multiplier = 10_f64.powi(exp * exp_sign);
                let magnitude = integer + fract;
                return Ok(JsonToken::Number(sign * exp_multiplier * magnitude));
            },
            b' ' | b'\t' | b'\n' | b'\r' | b','|b':' => {}, // ignore whitespace, commas, colons
            b't' => {
                return if check_slice(b"rue", parsing_index) { Ok(JsonToken::Boolean(true)) }
                else { Err(Error::new(InvalidData, "ERROR: Invalid token starting with 't'")) }
            },
            b'f' => {
                return if check_slice(b"alse", parsing_index) { Ok(JsonToken::Boolean(false)) } 
                else { Err(Error::new(InvalidData, "ERROR: Invalid token starting with 'f'")) }
            },
            b'n' => {
                return if check_slice(b"ull", parsing_index) { Ok(JsonToken::Null) }
                else { Err(Error::new(InvalidData, "ERROR: Invalid token starting with 'n'")) }
            },
            _ => { return Err(Error::new(InvalidData, "ERROR: Parsing invalid token.")); } 
        }
    }
    return Err(Error::new(InvalidData, "ERROR: Unexpectedly reached end of json file."));
}

pub struct JsonContext<'a> {
    parent: &'a JsonValue<'a>,
    child_index: usize
}

impl Json<'_> {

    pub fn get_root_context(&self) -> JsonContext {
        return JsonContext {
            parent: &self.root,
            child_index: 0
        }
    }

    pub fn open_collection<'a>(&self, element: &'a JsonValue<'a>) -> Option<JsonContext<'a>> {
       return match element {
            JsonValue::Object{ start_index, end_index: _ } => {
                Some(JsonContext {
                    parent: element,
                    child_index: *start_index
                })
            },
            JsonValue::Array{ start_index, end_index: _ } => {
                Some(JsonContext {
                    parent: element,
                    child_index: *start_index
                })
            },
            _ => None
        }
    }

    pub fn get_next_element(&self, context: &mut JsonContext) -> Option<(&[u8], &JsonValue)> {
        return match context.parent {
            JsonValue::Array { start_index: _, end_index } => {
                if context.child_index >= *end_index { None }
                else {
                    let value = &self.array_elements[context.child_index];
                    match value {
                        JsonValue::Array{ start_index: _, end_index } => {
                            context.child_index = *end_index;
                        },
                        _ => {
                            context.child_index += 1;
                        }
                    }
                    Some((b"", value))
                }
            },
            JsonValue::Object { start_index: _, end_index } => {
                if context.child_index >= *end_index { None }
                else {
                    let (name, value) = &self.object_members[context.child_index];
                    match value {
                        JsonValue::Object{ start_index: _, end_index } => {
                            context.child_index = *end_index;
                        },
                        _ => {
                            context.child_index += 1;
                        }
                    }
                    Some((name, value))
                }
            },
            _ => None
        }
    }
}
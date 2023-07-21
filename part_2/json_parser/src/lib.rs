use std::io::{Result, Error, ErrorKind::InvalidData};
use profiler::*;

// Note: This does not follow spec exactly.
// Known things that don't match spec:
//  - Root must be an object/array
//  - Anything following the root object/array is ignored
//  - Commas are ignored in general, so extra commas are allowed
//  - (Spec is uncertain on this but...) Keys in objects are not unique.
//      - HashMap/FxHashMap were slower than linear searching in my limited usecase.
//  - Strings are returned as just the bytes between two non-escaped quotes. Escape slashes and their following character are not removed.

#[derive(PartialEq, Debug)]
pub enum JsonValue<'a> {
    Object{ key_val_pairs: Vec<(&'a [u8], JsonValue<'a>)> }, // Note: "Keys" are not unique within a json object :(
    Array{ elements: Vec<JsonValue<'a>> },
    String(&'a [u8]),
    Number(f64),
    Boolean(bool),
    Null
}

enum JsonToken<'a> {
    ObjectStart, ObjectTerminate,
    ArrayStart, ArrayTerminate,
    String(&'a [u8]),
    Number(f64),
    Boolean(bool),
    Null
}

pub fn parse_json_bytes<'a>(json_bytes: &'a [u8]) -> Result<JsonValue<'a>> {
    time_function!();
    let mut index: usize = 0;
    match parse_token(&json_bytes, &mut index) {
        Ok(JsonToken::ObjectStart) => { return Ok(parse_json_object(json_bytes, &mut index)?); },
        Ok(JsonToken::ArrayStart) => { return Ok(parse_json_array(json_bytes, &mut index)?); },
        Ok(_) => { return Err(Error::new(InvalidData, "ERROR: Json must have an object or array as the root. Nothing may exist but whitespace after the root.")); },
        Err(e) => { return Err(e); }
    }
}

fn parse_json_object<'a>(json_bytes: &'a [u8], advancing_index: &mut usize) -> Result<JsonValue<'a>> {
    let mut members = Vec::<(&'a [u8], JsonValue<'a>)>::new();
    loop {
        match parse_token(json_bytes, advancing_index)? {
            JsonToken::String(key_str) => {
                match parse_token(json_bytes, advancing_index)? {
                    JsonToken::ObjectStart => { members.push((key_str, parse_json_object(json_bytes, advancing_index)?)); },
                    JsonToken::ArrayStart => { members.push((key_str, parse_json_array(json_bytes, advancing_index)?)); },
                    JsonToken::String(s) => { members.push((key_str, JsonValue::String(s))); },
                    JsonToken::Number(n) => { members.push((key_str, JsonValue::Number(n))); },
                    JsonToken::Boolean(b) => { members.push((key_str, JsonValue::Boolean(b))); },
                    JsonToken::Null => { members.push((key_str, JsonValue::Null)); },
                    JsonToken::ObjectTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected object member value but found end of object.")); },
                    JsonToken::ArrayTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected object member value but found end of array.")); }
                }
            },
            JsonToken::ObjectTerminate => { return Ok(JsonValue::Object{ key_val_pairs: members }); },
            _ => { return Err(Error::new(InvalidData, "ERROR: Expected a string for a key in a json object.")); }
        }
    }
}

fn parse_json_array<'a>(json_bytes: &'a [u8], advancing_index: &mut usize) -> Result<JsonValue<'a>> {
    let mut elements = Vec::<JsonValue<'a>>::new();
    loop {
        match parse_token(json_bytes, advancing_index)? {
            JsonToken::ObjectStart => { elements.push(parse_json_object(json_bytes, advancing_index)?); },
            JsonToken::ArrayStart => { elements.push(parse_json_array(json_bytes, advancing_index)?); },
            JsonToken::ArrayTerminate => { return Ok(JsonValue::Array{ elements: elements }); },
            JsonToken::String(s) => { elements.push(JsonValue::String(s)); },
            JsonToken::Number(n) => { elements.push(JsonValue::Number(n)); },
            JsonToken::Boolean(b) => { elements.push(JsonValue::Boolean(b)); },
            JsonToken::Null => { elements.push(JsonValue::Null); },
            JsonToken::ObjectTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected array element but found end of an object.")); }
        }
    }

}

fn parse_token<'a>(json_bytes: &'a [u8], advancing_index: &mut usize) -> Result<JsonToken<'a>> {

    while *advancing_index < json_bytes.len() {

        let check_slice = |s: &[u8], i: &mut usize| -> bool {
            let start_i = *i;
            *i += s.len();
            if json_bytes.len() < *i { return false; }
            return json_bytes[start_i..*i].eq(s);
        };

        let mut byte = json_bytes[*advancing_index];
        *advancing_index += 1; // index first, as it is expected to advance before returning from function
        match byte {
            b'{' => { return Ok(JsonToken::ObjectStart);},
            b'}' => { return Ok(JsonToken::ObjectTerminate); },
            b'[' => { return Ok(JsonToken::ArrayStart); },
            b']' => { return Ok(JsonToken::ArrayTerminate); },
            b'"' => {
                let start_str_index = *advancing_index;
                while *advancing_index < json_bytes.len() {
                    match json_bytes[*advancing_index] {
                        b'\\' => { *advancing_index += 2; }
                        b'"' => {
                            let end_quote_index = *advancing_index;
                            *advancing_index += 1;
                            return Ok(JsonToken::String(&json_bytes[start_str_index..end_quote_index]));
                        },
                        _ => { *advancing_index += 1; }
                    }
                }
            },
            b't' => {
                if check_slice(b"rue", advancing_index) { return Ok(JsonToken::Boolean(true)); }
                else { return Err(Error::new(InvalidData, "ERROR: Invalid token starting with 't'")); }
            },
            b'f' => {
                if check_slice(b"alse", advancing_index) { return Ok(JsonToken::Boolean(false)); } 
                else { return Err(Error::new(InvalidData, "ERROR: Invalid token starting with 'f'")); }
            },
            b'n' => {
                if check_slice(b"ull", advancing_index) { return Ok(JsonToken::Null); }
                else { return Err(Error::new(InvalidData, "ERROR: Invalid token starting with 'n'")); }
            },
            b'0'..=b'9'|b'-'|b'+' => {

                // pull sign
                let sign: f64;
                if byte == b'-' {
                    sign = -1.0;
                    byte = json_bytes[*advancing_index];
                    *advancing_index += 1;
                } else {
                    sign = 1.0;
                    if byte == b'+' { 
                        byte = json_bytes[*advancing_index];
                        *advancing_index += 1; 
                    }
                }

                // pull integer
                let mut integer: f64 = (byte - b'0') as f64;
                while *advancing_index < json_bytes.len() {
                    byte = json_bytes[*advancing_index];

                    match byte {
                        b'0'..=b'9' => {
                            integer = (integer * 10.0) + (byte - b'0') as f64;
                            *advancing_index += 1;
                        },
                        _ => {break;}
                    }
                }

                // pull fraction
                let mut fract: f64 = 0.0;
                if byte == b'.' {
                    *advancing_index += 1;
                    let mut fract_mul: f64 = 1.0 / 10.0;

                    while *advancing_index < json_bytes.len() {
                        byte = json_bytes[*advancing_index];
    
                        match byte {
                            b'0'..=b'9' => {
                                fract += fract_mul * (byte - b'0') as f64;
                                fract_mul *= 1.0 / 10.0;
                                *advancing_index += 1; 
                            },
                            _ => {break;}
                        }
                    }
                }

                // pull explicit exponent
                let mut exp: i32 = 0;
                let mut exp_sign: i32 = 1;
                if byte == b'e' || byte == b'E' {
                    *advancing_index += 1;

                    if json_bytes[*advancing_index] == b'-' {
                        exp_sign = -1;
                        *advancing_index += 1;
                    } else {
                        if byte == b'+' { 
                            *advancing_index += 1; 
                        }
                    }

                    while *advancing_index < json_bytes.len() {
                        byte = json_bytes[*advancing_index];
    
                        match byte {
                            b'0'..=b'9' => {
                                exp *= 10;
                                exp += (byte - b'0') as i32;
                                *advancing_index += 1; 
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
            _ => { return Err(Error::new(InvalidData, "ERROR: Parsing invalid token.")); } 
        }
    }
    return Err(Error::new(InvalidData, "ERROR: Unexpectedly reached end of json file."));
}
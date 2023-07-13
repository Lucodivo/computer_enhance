use std::io::{Result, Error, ErrorKind::InvalidData};

#[derive(PartialEq, Debug)]
pub enum JsonValue<'a> {
    Object{ key_val_pairs: Vec<(&'a str, JsonValue<'a>)> }, // Note: "Keys" are not unique within a json object :(
    Array{ elements: Vec<JsonValue<'a>> },
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

enum JsonToken<'a> {
    ObjectStart,
    ObjectTerminate,
    ArrayStart,
    ArrayTerminate,
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null
}

pub fn parse_json_str<'a>(json_str: &'a str) -> Result<JsonValue<'a>> {
    let json_bytes = json_str.as_bytes();
    
    let mut index = 0;
    match parse_token(json_str, &json_bytes, &mut index) {
        Ok(JsonToken::ObjectStart) => { return Ok(parse_json_object(json_str, &json_bytes, &mut index)?); },
        Ok(JsonToken::ArrayStart) => { return Ok(parse_json_array(json_str, &json_bytes, &mut index)?); },
        Ok(_) => { return Err(Error::new(InvalidData, "ERROR: Json must have an object or array as the root. Nothing may exist but whitespace after the root.")); },
        Err(e) => { return Err(e); }
    }
}

fn parse_json_object<'a>(json_str: &'a str, json_bytes: &[u8], advancing_index: &mut usize) -> Result<JsonValue<'a>> {
    let mut members = Vec::<(&'a str, JsonValue<'a>)>::new();

    let mut object_open = true;
    while object_open {
        let key_token = parse_token(json_str, json_bytes, advancing_index)?; // NOTE: does this create a local i? or does it modify the i in the parent scope?

        match key_token {
            JsonToken::String(key_str) => {
                let value_token = parse_token(json_str, json_bytes, advancing_index)?; 
                match value_token {
                    JsonToken::ObjectStart => { 
                        let object_value = parse_json_object(json_str, json_bytes, advancing_index)?;
                        members.push((key_str, object_value));
                    },
                    JsonToken::ArrayStart => {
                        let array_value = parse_json_array(json_str, json_bytes, advancing_index)?;
                        members.push((key_str, array_value));
                    },
                    JsonToken::String(s) => { members.push((key_str, JsonValue::String(s))); },
                    JsonToken::Number(n) => { members.push((key_str, JsonValue::Number(n))); },
                    JsonToken::Boolean(b) => { members.push((key_str, JsonValue::Boolean(b))); },
                    JsonToken::Null => { members.push((key_str, JsonValue::Null)); },
                    JsonToken::ObjectTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected object member value but found end of object.")); },
                    JsonToken::ArrayTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected object member value but found end of array.")); }
                }
            },
            JsonToken::ObjectTerminate => {
                object_open = false;
            },
            _ => {
                return Err(Error::new(InvalidData, "ERROR: Expected a string for a key in a json object."));
            }
        }
    }

    return Ok(JsonValue::Object{ key_val_pairs: members });
}

fn parse_json_array<'a>(json_str: &'a str, json_bytes: &[u8], advancing_index: &mut usize) -> Result<JsonValue<'a>> {
    let mut elements = Vec::<JsonValue<'a>>::new();

    let mut array_open = true;
    while array_open {
        let element_token = parse_token(json_str, json_bytes, advancing_index)?; // NOTE: does this create a local i? or does it modify the i in the parent scope?
        match element_token {
            JsonToken::ObjectStart => { 
                let object_value = parse_json_object(json_str, json_bytes, advancing_index)?;
                elements.push(object_value);
            },
            JsonToken::ArrayStart => {
                let array_value = parse_json_array(json_str, json_bytes, advancing_index)?;
                elements.push(array_value);
            },
            JsonToken::ArrayTerminate => { array_open = false; },
            JsonToken::String(s) => { elements.push(JsonValue::String(s)); },
            JsonToken::Number(n) => { elements.push(JsonValue::Number(n)); },
            JsonToken::Boolean(b) => { elements.push(JsonValue::Boolean(b)); },
            JsonToken::Null => { elements.push(JsonValue::Null); },
            JsonToken::ObjectTerminate => { return Err(Error::new(InvalidData, "ERROR: Expected array element but found end of an object.")); }
        }
    }

    return Ok(JsonValue::Array{ elements: elements });
}

fn parse_token<'a>(json_str: &'a str, json_bytes: &[u8], advancing_index: &mut usize) -> Result<JsonToken<'a>> {
    
    while *advancing_index < json_bytes.len() {
        let byte = json_bytes[*advancing_index];
        *advancing_index += 1;
        match byte {
            b'{' => { return Ok(JsonToken::ObjectStart);},
            b'}' => { return Ok(JsonToken::ObjectTerminate); },
            b'[' => { return Ok(JsonToken::ArrayStart); },
            b']' => { return Ok(JsonToken::ArrayTerminate); },
            b'"' => {
                let start_str_index = *advancing_index;
                let mut escaped = false;
                while *advancing_index < json_bytes.len() {
                    match json_bytes[*advancing_index] {
                        b'\\' => { escaped = true; }
                        b'"' if !escaped => {
                            let end_quote_index = *advancing_index;
                            *advancing_index += 1;
                            return Ok(JsonToken::String(&json_str[start_str_index..end_quote_index]));
                        },
                        _ => { escaped = false; }
                    }
                    *advancing_index += 1;
                }
                return Err(Error::new(InvalidData, "ERROR: Json ended before the end of a string.")); 
            },
            b't' => {
                if let Some(b'r') = json_bytes.get(*advancing_index) {
                    if let Some(b'u') = json_bytes.get(*advancing_index+1) {
                        if let Some(b'e') = json_bytes.get(*advancing_index+2) {
                            *advancing_index += 3;
                            return Ok(JsonToken::Boolean(true));
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Invalid token starting with 't'"));
            },
            b'f' => {
                if let Some(b'a') = json_bytes.get(*advancing_index) {
                    if let Some(b'l') = json_bytes.get(*advancing_index+1) {
                        if let Some(b's') = json_bytes.get(*advancing_index+2) {
                            if let Some(b'e') = json_bytes.get(*advancing_index+3) {
                                *advancing_index += 4;
                                return Ok(JsonToken::Boolean(true));
                            }
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Invalid token starting with 'f'"));
            },
            b'n' => {
                if let Some(b'u') = json_bytes.get(*advancing_index) {
                    if let Some(b'l') = json_bytes.get(*advancing_index+2) {
                        if let Some(b'l') = json_bytes.get(*advancing_index+3) {
                            *advancing_index += 4;
                            return Ok(JsonToken::Null);
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Invalid token starting with 'n'"));
            },
            b'0'..=b'9'|b'-'|b'+' => {
                let first_number_index = *advancing_index - 1;
                while *advancing_index < json_bytes.len() {
                    match json_bytes[*advancing_index] {
                        b'0'..=b'9'|b'.'|b'+'|b'-'|b'e'|b'E' => { *advancing_index += 1; },
                        _ => { break; }
                    }
                }
                if let Ok(number) = json_str[first_number_index..*advancing_index].parse::<f64>() {
                    return Ok(JsonToken::Number(number));
                }
                return Err(Error::new(InvalidData, "ERROR: Invalid number."));
            },
            b' ' | b'\t' | b'\n' | b'\r' | b','|b':' => {}, // ignore whitespace, commas, colons
            _ => { // anything else is invalid
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token.")); 
            } 
        }
    }

    return Err(Error::new(InvalidData, "ERROR: End of object or array not found."));
}


/* 
    ***************************
    ********** TESTS **********
    ***************************
*/

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_nothing() {
        let json_str: &'static str = "";
        parse_json_str(json_str).expect_err("Expected error when parsing nothing.");
    }

    #[test]
    fn parse_empty_object() {
        let json_str: &'static str = "{}";
        match parse_json_str(json_str) {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => { assert_eq!(pairs.len(), 0); },
            _ => { panic!("Expected empty object."); }
        }
    }

    #[test]
    fn parse_empty_array() {
        let json_str: &'static str = "[]";
        match parse_json_str(json_str) {
            Ok(JsonValue::Array{ elements: e }) => { assert_eq!(e.len(), 0); },
            _ => { panic!("Expected empty array."); }
        }
    }

    #[test]
    fn parse_object_w_empty_array() {
        let json_str: &'static str = "{\"empty_array\":[]}";
        let result = parse_json_str(json_str);
        match result {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => {
                assert_eq!(pairs.len(), 1);
                let (key, value) = &pairs[0];
                assert_eq!(key, &"empty_array");
                match value {
                    JsonValue::Array{ elements: e } => {
                        assert_eq!(e.len(), 0);
                    },
                    _ => { panic!("Expected empty array."); }
                }
            },
            _ => { panic!("Expected object with empty array."); }
        }
    }

    #[test]
    fn parse_pairs_test() {
        let json_str: &'static str = "\
            {\t\
                \"pairs\" : [\
                    {\
                        \"x0\":12.5,\
                        \"y0\":24.5,\
                        \"x1\":50.5,\
                        \"y1\":37.5\
                    },\
                    {\
                        \"x0\":12.25,\
                        \"y0\":22.25,\
                        \"x1\":-17.25,\
                        \"y1\":3.525e-9\
                    }\r\
                ]\n\
            }";
        
        let result = parse_json_str(json_str);
        match result {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => {
                assert_eq!(pairs.len(), 1);
                let (key, value) = &pairs[0];
                assert_eq!(key, &"pairs");
                match value {
                    JsonValue::Array{ elements: e } => {
                        assert_eq!(e.len(), 2);
                        match &e[0] {
                            JsonValue::Object{ key_val_pairs: pairs } => {
                                let key_pair_vals_1 = vec![("x0", JsonValue::Number(12.5f64)), 
                                                            ("y0", JsonValue::Number(24.5f64)),
                                                            ("x1", JsonValue::Number(50.5f64)), 
                                                            ("y1", JsonValue::Number(37.5f64))];
                                assert_eq!(pairs.len(), key_pair_vals_1.len());
                                assert!(pairs.iter().eq(key_pair_vals_1.iter()));
                            },
                            _ => { panic!("Expected object."); }
                        }
                        match &e[1] {
                            JsonValue::Object{ key_val_pairs: pairs } => {
                                let key_pair_vals_2 = vec![("x0", JsonValue::Number(12.25f64)), 
                                                            ("y0", JsonValue::Number(22.25f64)),
                                                            ("x1", JsonValue::Number(-17.25f64)), 
                                                            ("y1", JsonValue::Number(3.525e-9f64))];
                                assert_eq!(pairs.len(), key_pair_vals_2.len());
                                assert!(pairs.iter().eq(key_pair_vals_2.iter()));
                            },
                            _ => { panic!("Expected object."); }
                        }
                    },
                    _ => { panic!("Expected pairs array."); }
                }
            },
            _ => { panic!("Expected object with a pairs array."); }
        }
    }
}
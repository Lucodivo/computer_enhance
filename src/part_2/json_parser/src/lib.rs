//use std::fs;
use std::io::{Result, Error, ErrorKind::InvalidData};

#[derive(PartialEq, Debug)]
pub enum Value<'a> {
    ObjectOpen,
    ObjectClose,
    ArrayOpen,
    ArrayClose,
    Key(&'a str),
    String(&'a str),
    Number(f32),
    Boolean(bool),
    Null,
}

// TODO: Can't return a reference to json string that will go out of scope
// pub fn parse_json_file(json_filename: &str) -> Result<Vec<Value>> {
//     let json = fs::read_to_string(json_filename)?;
//     parse_json_str(&json)
// }

#[allow(dead_code)] // TODO: Remove this
fn parse_json_str<'a>(json_str: &'a str) -> Result<Vec<Value>> {
    let json_chars = json_str.as_bytes();
    let mut values: Vec<Value> = Vec::new();
    let mut i = 0;

    let mut inside_object_stack: Vec<bool> = Vec::new();

    while i < json_chars.len() {
        match json_chars[i] {
            b'{' => {
                values.push(Value::ObjectOpen);
                inside_object_stack.push(true);
                i += 1;
            },
            b'}' => {
                values.push(Value::ObjectClose);
                assert!(inside_object_stack.pop() == Some(true));
                i += 1;
            },
            b'[' => {
                values.push(Value::ArrayOpen);
                inside_object_stack.push(false);
                i += 1;
            },
            b']' => {
                values.push(Value::ArrayClose);
                assert!(inside_object_stack.pop() == Some(false));
                i += 1;
            },
            b'"' => {
                 let is_key = match values.last() {
                    Some(Value::Key(_)) => false,
                    _ => { 
                        inside_object_stack.last() == Some(&true)
                    }
                };
                let mut escaped = false;
                let open_quote_index = i;
                i += 1;
                while i < json_chars.len() {
                    match json_chars[i] {
                        b'\\' => { escaped = true; }
                        b'"' => {
                            if !escaped {
                                if is_key { 
                                    values.push(Value::Key(&json_str[open_quote_index+1..i])); 
                                } else { 
                                    values.push(Value::String(&json_str[open_quote_index+1..i])); 
                                }
                                i += 1;
                                break;
                            } else { escaped = false; }
                        },
                        _ => { escaped = false; }
                    }
                    i += 1;
                }
            },
            b't' => {
                if let Some(b'r') = json_chars.get(i+1) {
                    if let Some(b'u') = json_chars.get(i+2) {
                        if let Some(b'e') = json_chars.get(i+3) {
                            values.push(Value::Boolean(true));
                            i += 3;
                            continue;
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token 't'"));
            },
            b'f' => {
                if let Some(b'a') = json_chars.get(i+1) {
                    if let Some(b'l') = json_chars.get(i+2) {
                        if let Some(b's') = json_chars.get(i+3) {
                            if let Some(b'e') = json_chars.get(i+4) {
                                values.push(Value::Boolean(false));
                                i += 4;
                                continue;
                            }
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token 'f'"));
            },
            b'n' => {
                if let Some(b'u') = json_chars.get(i+1) {
                    if let Some(b'l') = json_chars.get(i+2) {
                        if let Some(b'l') = json_chars.get(i+3) {
                            values.push(Value::Null);
                            i += 3;
                            continue;
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token 'n'"));
            },
            b'0'..=b'9'|b'-'|b'+' => { // TODO: Numbers will need more validation. This is just grabbing characters in the value slot where a number should be.
                let first_number_index = i;
                i += 1;
                while i < json_chars.len() {
                    match json_chars[i] {
                        b'0'..=b'9'|b'.'|b'+'|b'-'|b'e'|b'E' => {},
                        _ => {
                            values.push(Value::Number(json_str[first_number_index..i].parse::<f32>().unwrap()));
                            break;
                        }
                    }
                    i += 1;
                }
            },
            b' ' | b'\t' | b'\n' | b'\r' | b','|b':' => { i += 1; }, // ignore whitespace, commas, colons
            _ => { 
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token")); 
            } // anything else is invalid
        }
    }

    assert!(inside_object_stack.len() == 0);

    Ok(values)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_nothing() {
        let json_str: &'static str = "";
        let result = parse_json_str(json_str);
        match result {
            Ok(empty_vec) => {
                assert_eq!(empty_vec.len(), 0);
            },
            Err(e) => { panic!("ERROR: {}", e); }
        }
    }

    #[test]
    fn parse_empty_object() {
        let json_str: &'static str = "{}";
        let result = parse_json_str(json_str);
        match result {
            Ok(empty_object) => {
                assert_eq!(empty_object.len(), 2);
                assert_eq!(empty_object[0], Value::ObjectOpen);
                assert_eq!(empty_object[1], Value::ObjectClose);
            },
            Err(e) => { panic!("ERROR: {}", e); }
        }
    }

    #[test]
    fn parse_empty_array() {
        let json_str: &'static str = "[]";
        let result = parse_json_str(json_str);
        match result {
            Ok(empty_array) => {
                assert_eq!(empty_array.len(), 2);
                assert_eq!(empty_array[0], Value::ArrayOpen);
                assert_eq!(empty_array[1], Value::ArrayClose);
            },
            Err(e) => { panic!("ERROR: {}", e); }
        }
    }

    #[test]
    fn parse_object_w_empty_array() {
        let json_str: &'static str = "{\"empty_array\":[]}";
        let result = parse_json_str(json_str);
        match result {
            Ok(object_w_empty_array) => {
                assert_eq!(object_w_empty_array.len(), 5);
                assert_eq!(object_w_empty_array[0], Value::ObjectOpen);
                assert_eq!(object_w_empty_array[1], Value::Key("empty_array"));
                assert_eq!(object_w_empty_array[2], Value::ArrayOpen);
                assert_eq!(object_w_empty_array[3], Value::ArrayClose);
                assert_eq!(object_w_empty_array[4], Value::ObjectClose);
            },
            Err(e) => { panic!("ERROR: {}", e); }
        }
    }

    #[test]
    fn parse_pairs_test() {
        let json_str: &'static str = 
            "{\"pairs\":[{\"x0\":12.5,\"y0\":24.5,\"x1\":50.5,\"y1\":37.5},{\"x0\":12.25,\"y0\":22.25,\"x1\":-17.25,\"y1\":3.525e-9}]}";
        let result = parse_json_str(json_str);
        match result {
            Ok(object_with_pairs) => {
                assert_eq!(object_with_pairs.len(), 25);
                assert_eq!(object_with_pairs[0], Value::ObjectOpen);
                assert_eq!(object_with_pairs[1], Value::Key("pairs"));
                assert_eq!(object_with_pairs[2], Value::ArrayOpen);
                assert_eq!(object_with_pairs[3], Value::ObjectOpen);
                assert_eq!(object_with_pairs[4], Value::Key("x0"));
                assert_eq!(object_with_pairs[5], Value::Number(12.5));
                assert_eq!(object_with_pairs[6], Value::Key("y0"));
                assert_eq!(object_with_pairs[7], Value::Number(24.5));
                assert_eq!(object_with_pairs[8], Value::Key("x1"));
                assert_eq!(object_with_pairs[9], Value::Number(50.5));
                assert_eq!(object_with_pairs[10], Value::Key("y1"));
                assert_eq!(object_with_pairs[11], Value::Number(37.5));
                assert_eq!(object_with_pairs[12], Value::ObjectClose);
                assert_eq!(object_with_pairs[13], Value::ObjectOpen);
                assert_eq!(object_with_pairs[14], Value::Key("x0"));
                assert_eq!(object_with_pairs[15], Value::Number(12.25));
                assert_eq!(object_with_pairs[16], Value::Key("y0"));
                assert_eq!(object_with_pairs[17], Value::Number(22.25));
                assert_eq!(object_with_pairs[18], Value::Key("x1"));
                assert_eq!(object_with_pairs[19], Value::Number(-17.25));
                assert_eq!(object_with_pairs[20], Value::Key("y1"));
                assert_eq!(object_with_pairs[21], Value::Number(3.525e-9));
                assert_eq!(object_with_pairs[22], Value::ObjectClose);
                assert_eq!(object_with_pairs[23], Value::ArrayClose);
                assert_eq!(object_with_pairs[24], Value::ObjectClose);
            },
            Err(e) => { panic!("ERROR: {}", e); }
        }
    }
}
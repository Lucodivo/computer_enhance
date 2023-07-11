//use std::fs;
use std::io::{Result, Error, ErrorKind::InvalidData};

#[derive(PartialEq, Debug)]
pub enum Value<'a> {
    Collection{ values: JsonCollection<'a> },
    Key(&'a str),
    String(&'a str),
    Number(f32),
    Boolean(bool),
    Null,
}

#[derive(PartialEq, Debug)]
pub struct JsonCollection<'a> {
    elements: Vec<Value<'a>>,
    key_val_pairs: bool,
}

// TODO: Can't return a reference to json string that will go out of scope
// pub fn parse_json_file(json_filename: &str) -> Result<Vec<Value>> {
//     let json = fs::read_to_string(json_filename)?;
//     parse_json_str(&json)
// }

fn parse_json_collection<'a>(json_str: &'a str, json_bytes: &'a [u8], index: usize, key_val_pairs: bool) -> Result<(Value<'a>, usize)> {
    let mut collection = JsonCollection{ elements: Vec::new(), key_val_pairs: key_val_pairs };
    
    let mut i = index;
    while i < json_bytes.len() {
        match json_bytes[i] {
            b'{' => {
                i += 1;
                match parse_json_collection(json_str, json_bytes, i, true) {
                    Ok((value, new_i)) => {
                        collection.elements.push(value);
                        i = new_i;
                    },
                    Err(e) => { return Err(e); }
                }
            },
            b'}' => {
                i += 1;
                let is_object = key_val_pairs;
                assert!(is_object);
                let value = Value::Collection{ values: collection };
                return Ok((value, i));
            },
            b'[' => {
                i += 1;
                match parse_json_collection(json_str, json_bytes, i, false) {
                    Ok((value, new_i)) => {
                        collection.elements.push(value);
                        i = new_i;
                    },
                    Err(e) => { return Err(e); }
                }
            },
            b']' => {
                i += 1;
                let is_array = !key_val_pairs;
                assert!(is_array);
                let value = Value::Collection{ values: collection };
                return Ok((value, i));
            },
            b'"' => {
                let is_key = if key_val_pairs { 
                    match collection.elements.last() {
                        Some(Value::Key(_)) => false,
                        _ => true
                    }
                } else { false };
                
                let mut escaped = false;
                let open_quote_index = i;
                i += 1;
                while i < json_bytes.len() {
                    match json_bytes[i] {
                        b'\\' => { escaped = true; }
                        b'"' => {
                            if !escaped {
                                if is_key { 
                                    collection.elements.push(Value::Key(&json_str[open_quote_index+1..i])); 
                                } else { 
                                    collection.elements.push(Value::String(&json_str[open_quote_index+1..i])); 
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
                if let Some(b'r') = json_bytes.get(i+1) {
                    if let Some(b'u') = json_bytes.get(i+2) {
                        if let Some(b'e') = json_bytes.get(i+3) {
                            collection.elements.push(Value::Boolean(true));
                            i += 3;
                            continue;
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token 't'"));
            },
            b'f' => {
                if let Some(b'a') = json_bytes.get(i+1) {
                    if let Some(b'l') = json_bytes.get(i+2) {
                        if let Some(b's') = json_bytes.get(i+3) {
                            if let Some(b'e') = json_bytes.get(i+4) {
                                collection.elements.push(Value::Boolean(false));
                                i += 4;
                                continue;
                            }
                        }
                    }
                }
                return Err(Error::new(InvalidData, "ERROR: Parsing invalid token 'f'"));
            },
            b'n' => {
                if let Some(b'u') = json_bytes.get(i+1) {
                    if let Some(b'l') = json_bytes.get(i+2) {
                        if let Some(b'l') = json_bytes.get(i+3) {
                            collection.elements.push(Value::Null);
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
                while i < json_bytes.len() {
                    match json_bytes[i] {
                        b'0'..=b'9'|b'.'|b'+'|b'-'|b'e'|b'E' => {},
                        _ => {
                            collection.elements.push(Value::Number(json_str[first_number_index..i].parse::<f32>().unwrap()));
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

    return Err(Error::new(InvalidData, "ERROR: End of object or array not found."));
}

#[allow(dead_code)] // TODO: Remove this
fn parse_json_str<'a>(json_str: &'a str) -> Result<Value> {
    let json_bytes = json_str.as_bytes();
    
    let mut root: Value = Value::Null;
    let mut i = 0;    
    while i < json_bytes.len() {
        match json_bytes[i] {
            b'{' => {
                i += 1;
                match parse_json_collection(json_str, &json_bytes, i, true) {
                    Ok((value, new_i)) => {
                        root = value;
                        i = new_i;
                    },
                    Err(e) => { return Err(e); }
                }
            }
            b'[' => {
                i += 1;
                match parse_json_collection(json_str, &json_bytes, i, false) {
                    Ok((value, new_i)) => {
                        root = value;
                        i = new_i;
                    },
                    Err(e) => { return Err(e); }
                }
            }
            b' ' | b'\t' | b'\n' | b'\r' => { i += 1; }, // ignore whitespace, commas, colons
            _ => { 
                return Err(Error::new(InvalidData, "ERROR: Json must have an object or array as the root. Nothing may exist but whitespace after the root.")); 
            }
        }
    }

    Ok(root)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_nothing() {
        let json_str: &'static str = "";
        match parse_json_str(json_str) {
            Ok(Value::Null) => {},
            _ => { panic!("ERROR: Expected null value"); }
        }
    }

    #[test]
    fn parse_empty_object() {
        let json_str: &'static str = "{}";
        match parse_json_str(json_str) {
            Ok(Value::Collection{ values: vals }) => {
                assert_eq!(vals.elements.len(), 0);
                assert_eq!(vals.key_val_pairs, true);
            },
            Err(e) => { panic!("ERROR: {}", e); }
            _ => { panic!("ERROR: Expected empty object."); }
        }
    }

    #[test]
    fn parse_empty_array() {
        let json_str: &'static str = "[]";
        match parse_json_str(json_str) {
            Ok(Value::Collection{ values: vals }) => {
                assert_eq!(vals.elements.len(), 0);
                assert_eq!(vals.key_val_pairs, false);
            },
            _ => { panic!("ERROR: Expected empty array."); }
        }
    }

    #[test]
    fn parse_object_w_empty_array() {
        let json_str: &'static str = "{\"empty_array\":[]}";
        match parse_json_str(json_str) {
            Ok(Value::Collection{ values: vals }) => {
                // assert root is an object that contains two elements (a empty array and its key)
                assert_eq!(vals.elements.len(), 2);
                assert_eq!(vals.key_val_pairs, true);
                assert_eq!(vals.elements[0], Value::Key("empty_array"));
                match &vals.elements[1] {
                    Value::Collection{ values: vals } => {
                        assert_eq!(vals.elements.len(), 0);
                        assert_eq!(vals.key_val_pairs, false);
                    },
                    _ => { panic!("ERROR: Expected empty array."); }
                }
            },
            _ => { panic!("ERROR: Expected empty object."); }
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
        
        let root = parse_json_str(json_str);
        match root {
            Ok(Value::Collection{ values: vals }) => {
                assert_eq!(vals.key_val_pairs, true); // collection is an object
                assert_eq!(vals.elements.len(), 2); // collection contains array and its key
                assert_eq!(vals.elements[0], Value::Key("pairs"));
                let pairs_array = &vals.elements[1];
                match pairs_array {
                    Value::Collection{ values: vals } => {
                        assert_eq!(vals.key_val_pairs, false); // collection is array
                        assert_eq!(vals.elements.len(), 2); // collection has two pairs
                        let pair_1 = &vals.elements[0];
                        match pair_1 {
                            Value::Collection{ values: vals } => {
                                assert_eq!(vals.key_val_pairs, true); // collection is object
                                assert_eq!(vals.elements.len(), 8); // collection has pairs of coordinates and keys
                                assert_eq!(vals.elements[0], Value::Key("x0"));
                                assert_eq!(vals.elements[1], Value::Number(12.5));
                                assert_eq!(vals.elements[2], Value::Key("y0"));
                                assert_eq!(vals.elements[3], Value::Number(24.5));
                                assert_eq!(vals.elements[4], Value::Key("x1"));
                                assert_eq!(vals.elements[5], Value::Number(50.5));
                                assert_eq!(vals.elements[6], Value::Key("y1"));
                                assert_eq!(vals.elements[7], Value::Number(37.5));
                            },
                            _ => { panic!("ERROR: Expected coordinate pair."); }
                        }
                        let pair_2 = &vals.elements[1];
                        match pair_2 {
                            Value::Collection{ values: vals } => {
                                assert_eq!(vals.key_val_pairs, true); // collection is object
                                assert_eq!(vals.elements.len(), 8); // collection has pairs of coordinates and keys
                                assert_eq!(vals.elements[0], Value::Key("x0"));
                                assert_eq!(vals.elements[1], Value::Number(12.25));
                                assert_eq!(vals.elements[2], Value::Key("y0"));
                                assert_eq!(vals.elements[3], Value::Number(22.25));
                                assert_eq!(vals.elements[4], Value::Key("x1"));
                                assert_eq!(vals.elements[5], Value::Number(-17.25));
                                assert_eq!(vals.elements[6], Value::Key("y1"));
                                assert_eq!(vals.elements[7], Value::Number(3.525e-9));
                            },
                            _ => { panic!("ERROR: Expected coordinate pair."); }
                        }
                    },
                    _ => { panic!("ERROR: Expected array of pairs"); }
                }
            },
            _ => { panic!("ERROR: Expected object with pairs array."); }
        }
    }
}
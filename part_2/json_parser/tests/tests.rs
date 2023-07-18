#[cfg(test)]
mod tests {

    use json_parser::{JsonValue, parse_json_bytes};

    #[test]
    fn parse_nothing() {
        let json_bytes: &'static [u8] = b"";
        parse_json_bytes(json_bytes).expect_err("Expected error when parsing nothing.");
    }

    #[test]
    fn parse_empty_object() {
        let json_bytes: &'static [u8] = b"{}";
        match parse_json_bytes(json_bytes) {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => { assert_eq!(pairs.len(), 0); },
            _ => { panic!("Expected empty object."); }
        }
    }

    #[test]
    fn parse_empty_array() {
        let json_bytes: &'static [u8] = b"[]";
        match parse_json_bytes(json_bytes) {
            Ok(JsonValue::Array{ elements: e }) => { assert_eq!(e.len(), 0); },
            _ => { panic!("Expected empty array."); }
        }
    }

    #[test]
    fn parse_object_w_empty_array() {
        let json_bytes: &'static [u8] = b"{\"empty_array\":[]}";
        let result = parse_json_bytes(json_bytes);
        match result {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => {
                assert_eq!(pairs.len(), 1);
                let (key, value) = &pairs[0];
                assert_eq!(key, &b"empty_array");
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
    fn parse_array_many_types() {
        let json_bytes: &'static [u8] = b"[\"string\", 12.5, 24, true, false, null]";
        let result = parse_json_bytes(json_bytes);
        match result {
            Ok(JsonValue::Array{ elements: e }) => {
                assert_eq!(e.len(), 6);
                match &e[0] {
                    JsonValue::String(s) => { assert_eq!(s, &b"string"); },
                    _ => { panic!("Expected string."); }
                }
                match &e[1] {
                    JsonValue::Number(n) => { assert_eq!(n, &12.5_f64); },
                    _ => { panic!("Expected number."); }
                }
                match &e[2] {
                    JsonValue::Number(n) => { assert_eq!(n, &24_f64); },
                    _ => { panic!("Expected number."); }
                }
                match &e[3] {
                    JsonValue::Boolean(b) => { assert_eq!(b, &true); },
                    _ => { panic!("Expected true."); }
                }
                match &e[4] {
                    JsonValue::Boolean(b) => { assert_eq!(b, &false); },
                    _ => { panic!("Expected false."); }
                }
                match &e[5] {
                    JsonValue::Null => {},
                    _ => { panic!("Expected null."); }
                }
            },
            _ => { panic!("Expected array with many types."); }
        }
    }

    #[test]
    fn parse_whitespace_and_escaped_characters() {
        let json_bytes: &'static [u8] = 
        b"  \t\n\r{\
            \t\n\r\"{}[]:true\\\"_\\\"false\\\'null123.01e-2\\\"\"  \t\n\r:\
                \t\n\r[  \t\n\r]\
        \t\n\r}  \t\n\r";
        
        let result = parse_json_bytes(json_bytes);
        match result {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => {
                assert_eq!(pairs.len(), 1);
                let (key, value) = &pairs[0];
                assert_eq!(key, &b"{}[]:true\\\"_\\\"false\\\'null123.01e-2\\\"");
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
        let json_bytes: &'static [u8] = b"\
            {\
                \"pairs\":[\
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
                    }\
                ]\
            }";
        
        let result = parse_json_bytes(json_bytes);
        match result {
            Ok(JsonValue::Object{ key_val_pairs: pairs }) => {
                assert_eq!(pairs.len(), 1);
                let (key, value) = &pairs[0];
                assert_eq!(key, &b"pairs");
                match value {
                    JsonValue::Array{ elements: e } => {
                        assert_eq!(e.len(), 2);
                        // TODO: Tests wouldn't print out very helpful error messages if these assertions fail. Consider refactor.
                        match &e[0] {
                            JsonValue::Object{ key_val_pairs: kvp_actual } => {
                                let kvp_expected: Vec<(&[u8], f64)> = 
                                    vec![(b"x0", 12.5), 
                                        (b"y0", 24.5),
                                        (b"x1", 50.5), 
                                        (b"y1", 37.5)];
                                assert_eq!(kvp_actual.len(), kvp_expected.len());
                                for kvp_expected in kvp_expected {
                                    match kvp_actual.iter().find(|&x| x.0 == kvp_expected.0) {
                                        Some((_, JsonValue::Number(actual_num) )) => {
                                            assert_eq!(*actual_num, kvp_expected.1);
                                        },
                                        _ => { panic!("Expected pair value for \"{:?}\" not found.", kvp_expected.0); }
                                    }
                                }
                            },
                            _ => { panic!("Expected object."); }
                        }
                        match &e[1] {
                            JsonValue::Object{ key_val_pairs: kvp_actual } => {
                                let kvp_expected: Vec<(&[u8], f64)> = 
                                    vec![(b"x0", 12.25), 
                                        (b"y0", 22.25),
                                        (b"x1", -17.25), 
                                        (b"y1", 3.525e-9)];
                                assert_eq!(kvp_expected.len(), kvp_actual.len());
                                for kvp_expected in kvp_expected {
                                    match kvp_actual.iter().find(|&x| x.0 == kvp_expected.0) {
                                        Some((_, JsonValue::Number(actual_num) )) => {
                                            assert_eq!(*actual_num, kvp_expected.1);
                                        },
                                        _ => { panic!("Expected pair value for \"{:?}\" not found.", kvp_expected.0); }
                                    }
                                }
                            },
                            _ => { panic!("Expected object."); }
                        }
                    },
                    _ => { panic!("Expected pairs array."); }
                }
            },
            Ok(_) => { panic!("Expected object with pairs array."); },
            Err(e) => { panic!("Error parsing json: {}", e); }
        }
    }
}
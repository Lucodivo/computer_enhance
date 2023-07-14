#[cfg(test)]
mod tests {

    use json_parser::{JsonValue, parse_json_str};

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
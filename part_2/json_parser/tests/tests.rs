#[macro_use]
extern crate matches;

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use json_parser::{JsonValue, parse_json_bytes};

    #[test]
    fn parse_nothing() {
        let json_bytes: &'static [u8] = b"";
        assert!(parse_json_bytes(json_bytes).is_err());
    }

    #[test]
    fn parse_empty_object() {
        let json_bytes: &'static [u8] = b"{}";
        let json = parse_json_bytes(json_bytes).expect("Failed to parse empty object.");
        let mut root_object = json.get_root_context();
        assert!(matches!(json.get_next_element(&mut root_object), None));
    }
    
    #[test]
    fn parse_empty_array() {
        let json_bytes: &'static [u8] = b"[]";
        let json = parse_json_bytes(json_bytes).expect("Failed to parse empty ARRAY.");
        let mut root_array = json.get_root_context();
        assert!(matches!(json.get_next_element(&mut root_array), None));
    }
    
    #[test]
    fn parse_object_w_empty_array() {
        let json_bytes: &'static [u8] = b"{\"empty_array\":[]}";
        let json = parse_json_bytes(json_bytes).expect("Failed to parse object w/ empty array.");
        let mut root_object = json.get_root_context();
        let empty_array = json.get_next_element(&mut root_object);
        if let Some((name, JsonValue::Array{ start_index, end_index })) = empty_array {
            assert_eq!(start_index, end_index); // assert that array is empty
            assert_eq!(name, b"empty_array");
        } else { panic!("Expected empty array."); }
    }
    
    #[test]
    fn parse_array_many_types() {
        let json_bytes: &'static [u8] = b"[\"string\", 12.5, 24, true, false, null]";
        let json = parse_json_bytes(json_bytes).expect("Failed to parse empty array with many types.");
        let mut root_array = json.get_root_context();
        if let Some((_, JsonValue::String(b"string"))) = json.get_next_element(&mut root_array){} else { panic!("Couldn't find string."); }
        if let Some((_, &JsonValue::Number(value))) = json.get_next_element(&mut root_array){
            assert_eq!(12.5f64, value);
        } 
        else { panic!("Couldn't find floating point."); }
        if let Some((_, &JsonValue::Number(value))) = json.get_next_element(&mut root_array){
            assert_eq!(24f64, value);
        } else { panic!("Couldn't find integer."); }
        if let Some((_, JsonValue::Boolean(true))) = json.get_next_element(&mut root_array){} else { panic!("Couldn't find true."); }
        if let Some((_, JsonValue::Boolean(false))) = json.get_next_element(&mut root_array){} else { panic!("Couldn't find false."); }
        if let Some((_, JsonValue::Null)) = json.get_next_element(&mut root_array){} else { panic!("Couldn't find null."); }
        assert!(matches!(json.get_next_element(&mut root_array), None));
    }
    
    #[test]
    fn parse_whitespace_and_escaped_characters() {
        let json_bytes: &'static [u8] = 
        b"  \t\n\r{\
            \t\n\r\"{}[]:true\\\"_\\\"false\\\'null123.01e-2\\\"\"  \t\n\r:\
            \t\n\r[  \t\n\r]\
        \t\n\r}  \t\n\r";
        let json = parse_json_bytes(json_bytes).expect("Failed to parse object w/ empty array.");
        let mut root_object = json.get_root_context();
        if let Some((name, JsonValue::Array{ start_index, end_index })) = json.get_next_element(&mut root_object) {
            assert_eq!(start_index, end_index); // assert that array is empty
            assert_eq!(name, b"{}[]:true\\\"_\\\"false\\\'null123.01e-2\\\"");
        } else { panic!("Expected empty array."); }
        assert!(matches!(json.get_next_element(&mut root_object), None));
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
        
        let json = parse_json_bytes(json_bytes).expect("Failed to parse object w/ empty array.");
        let mut root_object = json.get_root_context();
        if let Some((b"pairs", value)) = json.get_next_element(&mut root_object) {
            let mut pairs_array = json.open_collection(value).expect("Expected array.");
            if let Some((_, value)) = json.get_next_element(&mut pairs_array) {
                let mut pair_object = json.open_collection(value).expect("Expected point pair object.");
                if let Some((b"x0", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(12.5, value);
                } else { panic!(); }
                if let Some((b"y0", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(24.5, value);
                } else { panic!(); }
                if let Some((b"x1", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(50.5, value);
                } else { panic!(); }
                if let Some((b"y1", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(37.5, value);
                } else { panic!(); }
            } else { panic!("Expected two objects with array."); }
            if let Some((_, value)) = json.get_next_element(&mut pairs_array) {
                let mut pair_object = json.open_collection(value).expect("Expected point pair object.");
                if let Some((b"x0", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(12.25, value);
                } else { panic!(); }
                if let Some((b"y0", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(22.25, value);
                } else { panic!(); }
                if let Some((b"x1", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(-17.25, value);
                } else { panic!(); }
                if let Some((b"y1", &JsonValue::Number(value))) = json.get_next_element(&mut pair_object) {
                    assert_eq!(3.525e-9, value);
                } else { panic!(); }
            } else { panic!("Expected two objects with array."); }
            assert!(matches!(json.get_next_element(&mut pairs_array), None));
        } else { panic!("Expected object with array."); }
    }
}
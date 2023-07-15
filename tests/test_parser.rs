use serde_json::Value;

#[test]
fn test_from_slice() {
    let file = std::fs::read("./tests/sample.mcl").unwrap();

    let output = mcl::from_slice(&file).unwrap();

    assert!(output["jmmaa"] != Value::Null);
}

#[test]
fn test_from_str() {
    let output = mcl::from_str(r#"foo { bar "baz" }"#).unwrap();

    let num = &output["foo"]["bar"];

    assert!(num.is_string());

    let val = num.as_str();

    assert!(val == Some("baz"))
}

#[test]
fn test_arr() {
    let output = mcl::from_str("\"marky\" 32 23.23 null").unwrap();

    assert!(&output[0].is_string());
    assert!(&output[1].is_number());
    assert!(&output[2].is_number());
    assert!(&output[3].is_null());

    let val = output[2].as_f64();

    assert!(val == Some(23.23));
}

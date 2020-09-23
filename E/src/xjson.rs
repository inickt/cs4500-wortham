use serde_json::value::Value;
use serde_json::Deserializer;
use serde_json::json;
use serde_json::error::Category;
use serde::de::Deserialize;

pub fn xjson<R: std::io::Read>(reader: R) -> String {
    let mut input = Vec::default();

    // Loop until we hit an Eof or a json parsing error
    // Assume json is well formatted, but smoothly handle the case
    // where it is not (for convenience)
    let mut de = Deserializer::from_reader(reader);
    loop {
        match Value::deserialize(&mut de) {
            Ok(value) => input.push(value),
            Err(err) if err.classify() == Category::Eof => break,
            _ => {
                return String::from("Error in json formatting.");
            },
        }
    }

    let count = input.len();
    let first_line = json!({ "count": count, "seq": input.clone() });

    input.push(json!(count));
    input.reverse();
    let second_line = json!(input);

    format!("{}\n{}", first_line, second_line)
}

#[test]
fn test_empty() {
    assert_eq!(xjson("".as_bytes()), "{\"count\":0,\"seq\":[]}\n[0]");
}

#[test]
fn test_complex() {
    assert_eq!(xjson("1 null [1,2\n,3] {\"hello\":\"world\",\n\t \"n\": [false, [], 1.9932]} ".as_bytes()),
    "{\"count\":4,\"seq\":[1,null,[1,2,3],{\"hello\":\"world\",\"n\":[false,[],1.9932]}]}\
    \n[4,{\"hello\":\"world\",\"n\":[false,[],1.9932]},[1,2,3],null,1]");
}

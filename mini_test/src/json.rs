use serde_json::{Result, Value};
#[test]
fn test() -> Result<()> {
    let json = r#"
  {
    "name": "琼台博客",
    "age": 30,
    "blog": "https://www.qttc.net",
    "addr": null
  }"#;

    let v: Value = serde_json::from_str(json)?;

    println!("name = {}", v["name"]);
    println!("age = {}", v["age"]);
    println!("blog = {}", v["blog"]);
    println!("addr = {}", v["addr"]);

    Ok(())
}

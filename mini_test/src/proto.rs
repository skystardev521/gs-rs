/*
use no_proto::collection::{table::NP_Table,list::NP_List};
use no_proto::error::NP_Error;
use no_proto::NP_Factory;
use no_proto::json_flex::NP_JSON;

#[test]
fn xxxx(){
    //proto();
}

#[test]
    fn it_works() -> core::result::Result<(), NP_Error> {

        let factory: NP_Factory = NP_Factory::new(r#"{
            "type": "list",
            "of": {
                "type": "table",
                "columns": [
                    ["name", {"type": "string", "default": "no name"}],
                    ["age",  {"type": "i16", "default": 10}]
                ]
            }
        }"#)?;

        let mut new_buffer = factory.empty_buffer(None, None);

        new_buffer.open::<NP_List<NP_Table>>(&mut |_list| {

            Ok(())
        })?;

        new_buffer.deep_set("10.name", "something".to_owned())?;
        new_buffer.deep_set("10.name", "someth\"ing22".to_owned())?;
        new_buffer.deep_set("9.age", -29383i16)?;

        println!("Size: {:?}", new_buffer.calc_bytes()?);
        // new_buffer.compact(None, None)?;
        println!("Size: {:?}", new_buffer.calc_bytes()?);

         println!("JSON: {}", new_buffer.json_encode().stringify());
         new_buffer.compact(None, None)?;

        let value = new_buffer.deep_get::<NP_JSON>("9")?;

        new_buffer.

        println!("name: {}", value.unwrap().stringify());

        println!("BYTES: {:?}", new_buffer.close());

        // let buffer2 = factory.deep_set::<String>(return_buffer, "15", "hello, world".to_owned())?;

        // println!("value {:?}", factory.deep_get::<String>(return_buffer, "10.name")?);

        Ok(())
    }
    */

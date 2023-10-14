#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_binary, Addr};
    use serde_cw_value::{to_value, Value};

    use crate::JSON;

    #[test]
    fn test_vector_get() {
        // Parse JSON string with a vector
        let json_str = r#"{
            "data": [10, 20, 30]
        }"#;
        let parsed_json = JSON::from(json_str);

        // Get a vector value
        let data = parsed_json.get("data").unwrap();
        assert_eq!(
            data,
            &Value::Seq(vec![Value::U64(10), Value::U64(20), Value::U64(30)])
        );
    }

    #[test]
    fn test_vector_update() {
        // Parse JSON string with a vector
        let json_str = r#"{
            "data": [10, 20, 30]
        }"#;
        let mut parsed_json = JSON::from(json_str);

        // Update a vector value
        let new_data = Value::Seq(vec![Value::U64(40), Value::U64(50)]);
        parsed_json.update("data", new_data).unwrap();

        // Get the updated vector value
        let updated_data = parsed_json.get("data").unwrap();
        assert_eq!(
            updated_data,
            &Value::Seq(vec![Value::U64(40), Value::U64(50)])
        );
    }

    #[test]
    fn test_nested_array_update() {
        // Parse JSON string with an array of nested objects
        let json_str = r#"
        {
            "data": [
                {
                    "numbers": [10, 20, 30]
                },
                {
                    "numbers": [40, 50, 60]
                }
            ]
        }"#;

        let mut parsed_json = JSON::from(json_str);

        // Get the array of nested objects
        let array_value = parsed_json.get("data").unwrap();
        assert_eq!(
            array_value,
            &Value::Seq(vec![
                Value::Map({
                    let mut map = BTreeMap::new();
                    map.insert(
                        Value::String("numbers".to_string()),
                        Value::Seq(vec![Value::U64(10), Value::U64(20), Value::U64(30)]),
                    );
                    map
                }),
                Value::Map({
                    let mut map = BTreeMap::new();
                    map.insert(
                        Value::String("numbers".to_string()),
                        Value::Seq(vec![Value::U64(40), Value::U64(50), Value::U64(60)]),
                    );
                    map
                })
            ])
        );
        // assert_eq!(parsed_json.to_json_string().unwrap(), json_str);
        println!("{parsed_json:?}");

        // Update the array value in the nested object at index 1
        let new_value = Value::Seq(vec![Value::U64(70), Value::U64(80)]);
        parsed_json
            .update("data.1.numbers", new_value.clone())
            .unwrap();
        println!("{parsed_json:?}");

        // Get the updated array value in the nested object at index 1
        let updated_array_value = parsed_json.get("data.1.numbers").unwrap();
        assert_eq!(updated_array_value, &new_value);

        parsed_json
            .update("data.1.numbers.1", Value::U64(10))
            .unwrap();
        println!("{parsed_json:?}");

        // Get the updated array value in the nested object at index 1
        let updated_array_value = parsed_json.get("data.1.numbers.1").unwrap();
        assert_eq!(updated_array_value, &Value::U64(10));
    }

    #[test]
    fn test_binary_update() {
        #[cw_serde]
        struct Dummy {
            pub address: Addr,
            pub owner: Addr,
        }
        let mut data = Dummy {
            owner: Addr::unchecked("owner"),
            address: Addr::unchecked("address"),
        };
        let encoded = to_binary(&data).unwrap();

        // Start parsing
        let mut parsed_json = JSON::from(encoded);

        // Update a vector value
        parsed_json
            .update("owner", to_value(Addr::unchecked("new_owner")).unwrap())
            .unwrap();

        // Get the updated vector value
        let updated_data: Addr = parsed_json
            .get("owner")
            .unwrap()
            .clone()
            .deserialize_into()
            .unwrap();
        assert_eq!(updated_data, Addr::unchecked("new_owner"));

        data.owner = Addr::unchecked("new_owner");
        let encoded = to_binary(&data).unwrap();
        let json_encoded = to_binary(&parsed_json).unwrap();
        assert_eq!(encoded, json_encoded);

        // Also check deserialized comparision fro the values
        assert_eq!(data, parsed_json.to_any());
    }
}

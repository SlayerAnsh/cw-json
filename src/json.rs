use std::collections::BTreeMap;

use cosmwasm_std::{from_binary, Binary};
use serde::{Deserialize, Serialize};
use serde_cw_value::{to_value, Value};
use serde_json_wasm::{from_str, to_string};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSON(Value);

impl From<&str> for JSON {
    fn from(json_str: &str) -> Self {
        let parsed_json: Value = from_str(json_str).unwrap();
        Self(parsed_json)
    }
}

impl From<Binary> for JSON {
    fn from(binary: Binary) -> Self {
        let parsed_json: Value = from_binary(&binary).unwrap();
        Self(parsed_json)
    }
}

impl From<Value> for JSON {
    fn from(value: Value) -> Self {
        let v = Self(value);
        // Parse and store again to properly deserialize
        let parsed: String = v.into();
        Self::from(parsed.as_str())
    }
}

impl Into<String> for JSON {
    fn into(self) -> String {
        to_string(&self.0).unwrap()
    }
}

impl JSON {
    #[inline(never)]
    pub fn from_any<T: serde::ser::Serialize>(value: T) -> Self {
        Self::from(to_value(value).unwrap())
    }

    #[inline(always)]
    pub fn to_any<T: serde::de::DeserializeOwned>(self) -> T {
        self.0.deserialize_into().unwrap()
    }

    #[inline(always)]
    pub fn get<'a>(&'a self, key: &'a str) -> Option<&'a Value> {
        self.get_nested(&self.0, key.split('.'))
    }

    fn get_nested<'a, 'b, I>(&'a self, json: &'b Value, mut keys: I) -> Option<&'a Value>
    where
        'b: 'a,
        I: Iterator<Item = &'a str>,
    {
        match json {
            Value::Map(map) => {
                if let Some(key) = keys.next() {
                    map.get(&to_value(key).unwrap())
                        .and_then(|next_json| self.get_nested(next_json, keys))
                } else {
                    Some(json)
                }
            }
            Value::Seq(list) => {
                if let Some(index) = keys.next() {
                    if let Ok(idx) = index.parse::<usize>() {
                        list.get(idx)
                            .and_then(|next_json| self.get_nested(next_json, keys))
                    } else {
                        None
                    }
                } else {
                    Some(json)
                }
            }
            _ => match keys.next() {
                Some(_) => None,
                None => Some(json),
            },
        }
    }

    #[inline(always)]
    pub fn update(&mut self, key: &str, value: Value) -> Result<&Self, serde_json_wasm::de::Error> {
        let keys: Vec<&str> = key.split('.').collect();
        // Perform the update
        Self::update_nested(&mut self.0, &keys, value)?;
        Ok(self)
    }

    fn update_nested(
        json: &mut Value,
        keys: &[&str],
        value: Value,
    ) -> Result<(), serde_json_wasm::de::Error> {
        if let Some(current_key) = keys.first() {
            match json {
                Value::Map(ref mut map) => {
                    if keys.len() == 1 {
                        map.insert(Value::String(current_key.to_string()), value);
                    } else {
                        let next_json = map
                            .entry(Value::String(current_key.to_string()))
                            .or_insert(Value::Map(BTreeMap::new()));

                        Self::update_nested(next_json, &keys[1..], value)?;
                    };
                }
                Value::Seq(ref mut list) => {
                    if let Ok(index) = current_key.parse::<usize>() {
                        if keys.len() == 1 {
                            if index < list.len() {
                                list[index] = value;
                            } else {
                                return Err(serde_json_wasm::de::Error::Custom(format!(
                                    "Array index out of bounds: {}",
                                    index
                                )));
                            }
                        } else if index < list.len() {
                            Self::update_nested(&mut list[index], &keys[1..], value)?;
                        } else {
                            return Err(serde_json_wasm::de::Error::Custom(format!(
                                "Array index out of bounds: {}",
                                index
                            )));
                        };
                    } else {
                        return Err(serde_json_wasm::de::Error::Custom(format!(
                            "Invalid array index: {}",
                            current_key
                        )));
                    }
                }
                _ => {
                    // Handle other cases here if needed
                    return Err(serde_json_wasm::de::Error::Custom(format!(
                        "Invalid JSON structure at key: {}",
                        current_key
                    )));
                }
            }
        }
        Ok(())
    }
}

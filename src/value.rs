use std::error::Error;
use std::str;
use std::iter::FromIterator;

pub use map::Map;
use map::Entry;

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    String(String),
    Object(Map<String, Value>),
}

impl Value {
    /// Returns true if the `Value` is an Object. Returns false otherwise.
    ///
    /// For any Value on which `is_object` returns true, `as_object` and
    /// `as_object_mut` are guaranteed to return the map representation of the
    /// object.
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// If the `Value` is an Object, returns the associated Map. Returns None
    /// otherwise.
    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match *self {
            Value::Object(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match *self {
            Value::Object(ref mut map) => Some(map),
            _ => None,
        }
    }
}

impl str::FromStr for Value {
    type Err = Box<Error>;
    fn from_str(s: &str) -> Result<Value, Box<Error>> {
        let git_configs = Vec::from_iter(s.split("\0").map(String::from));
        let mut map = Map::new();

        for git_config in &git_configs {
            if git_config.is_empty() {
                continue;
            }
            let (keys, value) = split_once(git_config);
            if keys.len() == 0 {
                continue;
            }
            let split_keys = Vec::from_iter(keys.split(".").map(String::from));
            match split_keys.len() {
                1 => {
                    map.insert(split_keys[0].to_owned(), Value::String(value.to_owned()));
                    ()
                }
                2 => {
                    // TODO: split_keys[0].clone() why clone??
                    match map.entry(split_keys[0].clone()) {
                        Entry::Occupied(mut occupied) => {
                            occupied.get_mut().as_object_mut().unwrap().insert(
                                split_keys[1]
                                    .to_owned(),
                                Value::String(
                                    value.to_owned(),
                                ),
                            );
                            ()
                        }
                        Entry::Vacant(vacant) => {
                            let mut internal = Map::new();
                            internal.insert(
                                split_keys[1].to_owned(),
                                Value::String(value.to_owned()),
                            );
                            vacant.insert(Value::Object(internal));
                            ()
                        }
                    }
                }
                n if n >= 3 => {
                    // TODO: split_keys[0].clone() why clone??
                    match map.entry(split_keys[0].clone()) {
                        Entry::Occupied(mut occupied) => {
                            match occupied.get_mut().as_object_mut().unwrap().entry(
                                split_keys
                                    [1..n - 1]
                                    .join("."),
                            ) {
                                Entry::Occupied(mut occupied2) => {
                                    occupied2.get_mut().as_object_mut().unwrap().insert(
                                        split_keys[n - 1]
                                            .to_owned(),
                                        Value::String(
                                            value.to_owned(),
                                        ),
                                    );
                                    ()
                                }
                                Entry::Vacant(vacant2) => {
                                    let mut internal = Map::new();
                                    internal.insert(
                                        split_keys[n - 1].to_owned(),
                                        Value::String(value.to_owned()),
                                    );
                                    vacant2.insert(Value::Object(internal));
                                    ()
                                }
                            }
                        }
                        Entry::Vacant(vacant) => {
                            let mut internal = Map::new();
                            internal.insert(
                                split_keys[n - 1].to_owned(),
                                Value::String(value.to_owned()),
                            );
                            let mut external = Map::new();
                            external.insert(
                                split_keys[1..n - 1].join("."),
                                Value::Object(internal),
                            );
                            vacant.insert(Value::Object(external));
                            ()
                        }
                    }
                }
                _ => return Err(From::from("unexpected something happens.".to_owned())),
            }
        }

        Ok(Value::Object(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_empty() {
        let actual = "";
        let expected = Value::Object(Map::new());
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_empty2() {
        let actual = "\n";
        let expected = Value::Object(Map::new());
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_empty3() {
        let actual = "\n\n\n\n\n";
        let expected = Value::Object(Map::new());
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_key_and_empty_value() {
        let actual = "key\n";
        let mut internal = Map::new();
        internal.insert("key".to_owned(), Value::String("".to_owned()));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_one_key_and_value() {
        let actual = "key\nvalue\nvalue";
        let mut internal = Map::new();
        internal.insert("key".to_owned(), Value::String("value\nvalue".to_owned()));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_2config_key_and_value() {
        let actual = "key1\nvalue1\nvalue2\0";
        let mut internal = Map::new();
        internal.insert(
            "key1".to_owned(),
            Value::String("value1\nvalue2".to_owned()),
        );
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    //    #[test]
    //    fn parse_2config_key_and_value2() {
    //        let actual = "key1\nvalue1\nvalue2\0key2";
    //        let mut internal = Map::new();
    //        internal.insert("key1".to_owned(), Value::String("value1\nvalue2".to_owned()));
    //        let expected = Value::Object(internal);
    //        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    //    }

    #[test]
    fn parse_2config_key_and_value3() {
        let actual = "key1\nvalue1\nvalue2\0key2\nvalue3";
        let mut internal = Map::new();
        internal.insert(
            "key1".to_owned(),
            Value::String("value1\nvalue2".to_owned()),
        );
        internal.insert("key2".to_owned(), Value::String("value3".to_owned()));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_2keys_and_value1() {
        let actual = "key1.key2\nvalue1\nvalue2";
        let mut internal = Map::new();
        let mut internal2 = Map::new();
        internal2.insert(
            "key2".to_owned(),
            Value::String("value1\nvalue2".to_owned()),
        );
        internal.insert("key1".to_owned(), Value::Object(internal2));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_key_and_value2() {
        let actual = "key1\nvalue1\nvalue2";
        let mut internal = Map::new();
        match internal.entry("key1") {
            Entry::Occupied(_) => unimplemented!(),
            Entry::Vacant(vacant) => {
                vacant.insert(Value::String("value1\nvalue2".to_owned()));
                ()
            }
        }
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_2keys_and_value2() {
        let actual = "key1.key2\nvalue1\nvalue2";
        let mut internal = Map::new();
        match internal.entry("key1") {
            Entry::Occupied(_) => unimplemented!(),
            Entry::Vacant(vacant) => {
                let mut internal2 = Map::new();
                internal2.insert(
                    "key2".to_owned(),
                    Value::String("value1\nvalue2".to_owned()),
                );
                vacant.insert(Value::Object(internal2));
            }
        }
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_3keys_and_value1() {
        let actual = "key1.key2.key3\nvalue1\nvalue2";
        let mut internal = Map::new();
        let mut internal2 = Map::new();
        let mut internal3 = Map::new();
        internal3.insert(
            "key3".to_owned(),
            Value::String("value1\nvalue2".to_owned()),
        );
        internal2.insert("key2".to_owned(), Value::Object((internal3)));
        internal.insert("key1".to_owned(), Value::Object(internal2));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }

    #[test]
    fn parse_3keys_and_value2() {
        let actual = "key1.key2.key3\nvalue1\nvalue2\0key1.key2.key4\nvalue3\nvalue4";
        let mut internal = Map::new();
        let mut internal2 = Map::new();
        let mut internal3 = Map::new();
        internal3.insert(
            "key3".to_owned(),
            Value::String("value1\nvalue2".to_owned()),
        );
        internal3.insert(
            "key4".to_owned(),
            Value::String("value3\nvalue4".to_owned()),
        );
        internal2.insert("key2".to_owned(), Value::Object((internal3)));
        internal.insert("key1".to_owned(), Value::Object(internal2));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }
    #[test]
    fn parse_5keys_and_value2() {
        let actual = "key1.key2.key3.key4.key5\nvalue1
value2\0key1.key2.key3.key4.key6\nvalue3\nvalue4";
        let mut internal = Map::new();
        let mut internal2 = Map::new();
        let mut internal3 = Map::new();
        internal3.insert(
            "key5".to_owned(),
            Value::String("value1\nvalue2".to_owned()),
        );
        internal3.insert(
            "key6".to_owned(),
            Value::String("value3\nvalue4".to_owned()),
        );
        internal2.insert("key2.key3.key4".to_owned(), Value::Object((internal3)));
        internal.insert("key1".to_owned(), Value::Object(internal2));
        let expected = Value::Object(internal);
        assert_eq!(actual.parse::<Value>().unwrap(), expected);
    }
}

fn split_once(in_string: &str) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, "\n");
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}

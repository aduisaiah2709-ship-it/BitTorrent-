use std::{collections::BTreeMap, io::Read};

enum BValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(BTreeMap<Vec<u8>, BValue>),
}
enum State {
    None,
    Int,
    // if its false or none its still parsing the length, if not, its parsing the value.
    Bytes(Option<i32>),
    List,
    Dict,
}
pub struct Bencode {
    file_path: String,
}

impl Bencode {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
    pub fn decode(&self) -> Result<(), std::io::Error> {
        let mut _input = std::fs::read(&self.file_path)?;
        let mut input = b"i42e4:hell".to_vec();
        let mut position = 0;
        let mut width = 4;
        let mut state = State::None;
        let mut value: Option<BValue> = None;

        while position != input.len() {
            let end = position + width;
            let mut temp = &input[position..end];

            for index in 0..temp.len() {
                let byte = temp[index];
                match &state {
                    State::None => {
                        if byte == b'i' {
                            state = State::Int;
                            continue;
                        } else if byte.is_ascii_digit() {
                            state = State::Bytes(None);
                        } else {
                        }
                    }
                    State::Int => {
                        if byte == b'e' {
                            let mut number = &temp[1..index];
                            let number = std::str::from_utf8(number)
                                .unwrap()
                                .parse::<i32>()
                                .expect("not a number");
                            println!("{number:?}");
                            position += index + 1;
                            state = State::None;
                            width = 1;
                            break;
                        } else {
                            // println!("-{byte}-{index}-{position}-{end}-{}", temp.len());

                            continue;
                        }
                    }
                    State::Bytes(v) => {
                        if v.is_none() {
                            if byte != b':' {
                                println!("hello");
                                continue;
                            } else {
                                let length = &temp[0..index];
                                let number = std::str::from_utf8(length)
                                    .unwrap()
                                    .parse::<i32>()
                                    .expect("not a number");
                                println!("{number:?}");
                                position += index + 1;
                                width = 1;
                                state = State::Bytes(Some(number));
                                break;
                            }
                        } else {
                            let length = v.unwrap();
                            if index + 1 == length as usize {
                                let string = std::str::from_utf8(&temp[0..=index]).unwrap();
                                println!("{string}");
                                position += index + 1;
                                width = 1;
                                state = State::None;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if position + width != input.len() {
                width += 1;
            }
        }
        Ok(())
    }
}

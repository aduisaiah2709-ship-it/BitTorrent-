use std::{collections::BTreeMap, io::Read};

enum BValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(BTreeMap<Vec<u8>, BValue>),
}
#[derive(Debug)]
enum State {
    None,
    Int,
    // if its false or none its still parsing the length, if not, its parsing the value.
    Bytes(Option<i32>),

    // it can be an integer or a byte or another list? not sure about another list or a dict
    List(Box<ListState>),
    Dict,
}
#[derive(Debug)]
enum ListState {
    None,
    Int,
    Bytes(Option<i32>),
    List(Option<State>),
}
enum Either<T> {
    Left(T),
    Right(T),
}
#[derive(Debug)]
struct ReturnType<T> {
    width: usize,
    position: usize,
    should_break: bool,
    state: State,
    value: T,
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

    // return the new position and the new width IF it matches, else return the old one,
    // also the boolean is an indicator of whether to break the loop or continue
    // and also the state is the state
    // and also the last value is the actual value

    fn handle_int(
        &self,
        byte: u8,
        index: usize,
        position: usize,
        width: usize,
        temp: &[u8],
        new_state: State,
        old_state: State,
        is_list: bool,
    ) -> ReturnType<i64> {
        if byte == b'e' {
            let mut start = 1;
            if is_list {
                start = 2;
            }
            let mut number = &temp[start..index];
            println!("{number:?}");
            let number = std::str::from_utf8(number)
                .unwrap()
                .parse::<i64>()
                .expect("not a number");

            return ReturnType {
                width: 1,
                position: position + index + 1,
                should_break: true,
                state: new_state,
                value: number,
            };
        } else {
            return ReturnType {
                width,
                position,
                should_break: false,
                state: old_state,
                value: 0,
            };
        }
    }
    fn handle_byte(
        &self,
        state_byte: Option<i32>,
        byte: u8,
        index: usize,
        position: usize,
        width: usize,

        temp: &[u8],
    ) -> ReturnType<String> {
        let mut return_type = ReturnType {
            width,
            position,
            should_break: false,
            state: State::Bytes(None),
            value: String::new(),
        };
        if state_byte.is_none() {
            if byte != b':' {
                return_type.should_break = false;
                return return_type;
            } else {
                let length = &temp[0..index];
                let number = std::str::from_utf8(length)
                    .unwrap()
                    .parse::<i32>()
                    .expect("not a number");
                println!("{number:?}");
                return_type.position += index + 1;
                return_type.width = 1;
                return_type.state = State::Bytes(Some(number));
                return_type.should_break = true;
            }
        } else {
            let length = state_byte.unwrap();
            if index + 1 == length as usize {
                let string = std::str::from_utf8(&temp[0..=index]).unwrap();
                println!("{string}");
                return_type.value = string.to_string();
                return_type.position += index + 1;
                return_type.width = 1;
                return_type.state = State::None;
                return_type.should_break = true;
            }
        }

        return_type
    }
    fn handle_list(
        state_list: Option<Vec<State>>,
        byte: u8,
        index: usize,
        position: usize,
        width: usize,
        temp: &[u8],
    ) {
    }
    pub fn decode(&self) -> Result<(), std::io::Error> {
        let mut _input = std::fs::read(&self.file_path)?;
        // i42e4:hellli97ee
        let mut input = b"4:helli42eli97ei47ei87ei98ei78882920ee".to_vec();
        let mut position = 0;
        let mut width = 4;
        let mut state = State::None;
        let mut value: Option<BValue> = None;
        let mut start_of_list = true;
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
                        } else if byte == b'l' {
                            state = State::List(Box::new(ListState::None));
                        } else {
                        }
                    }
                    State::Int => {
                        let result = self.handle_int(
                            byte,
                            index,
                            position,
                            width,
                            temp,
                            State::None,
                            State::Int,
                            false,
                        );

                        if result.should_break {
                            println!("{}", result.value);
                            width = result.width;
                            position = result.position;
                            state = result.state;
                            break;
                        } else {
                            continue;
                        }
                    }
                    State::Bytes(v) => {
                        let result = self.handle_byte(*v, byte, index, position, width, temp);

                        if result.should_break {
                            width = result.width;
                            position = result.position;
                            state = result.state;
                            break;
                        } else {
                            continue;
                        }
                    }
                    State::List(v) => match **v {
                        ListState::None => {
                            if byte == b'i' {
                                state = State::List(Box::new(ListState::Int));
                                continue;
                            }
                            if byte.is_ascii_digit() {
                                state = State::List(Box::new(ListState::Bytes(None)))
                            }
                        }
                        ListState::Int => {
                            let addition = if start_of_list { 1 } else { 0 };
                            let result = self.handle_int(
                                byte,
                                index,
                                position + addition,
                                width,
                                temp,
                                State::List(Box::new(ListState::None)),
                                State::List(Box::new(ListState::Int)),
                                start_of_list,
                            );

                            if result.should_break {
                                println!("{}", result.value);
                                width = result.width;
                                position = result.position;

                                state = result.state;
                                start_of_list = false;
                                break;
                            } else {
                                continue;
                            }
                        }
                        _ => {}
                    },
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

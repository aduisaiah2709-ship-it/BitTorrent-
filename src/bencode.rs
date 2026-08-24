use std::{collections::BTreeMap, io::Read};

#[derive(Debug)]
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
    End,
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

            let number = std::str::from_utf8(number)
                .unwrap()
                .parse::<i64>()
                .expect("not a number");

            return ReturnType {
                width: 1,
                position: position + index + if is_list { 0 } else { 1 },
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
        is_list: bool,
        is_static_list: bool,
    ) -> ReturnType<Vec<u8>> {
        let mut start = 0;
        if is_list {
            start = 1;
        }
        let mut return_type: ReturnType<Vec<u8>> = ReturnType {
            width,
            position,
            should_break: false,
            state: if is_static_list {
                State::List(Box::new(ListState::Bytes(None)))
            } else {
                State::Bytes(None)
            },
            value: Vec::new(),
        };
        if state_byte.is_none() {
            if byte != b':' {
                return_type.should_break = false;

                return return_type;
            } else {
                let length = &temp[start..index];

                let number = std::str::from_utf8(length)
                    .unwrap()
                    .parse::<i32>()
                    .expect("not a number");

                return_type.position += index + 1;
                return_type.width = 1;
                return_type.state = if is_static_list {
                    State::List(Box::new(ListState::Bytes(Some(number))))
                } else {
                    State::Bytes(Some(number))
                };
                return_type.should_break = true;
            }
        } else {
            let length = state_byte.unwrap();
            if index + 1 == length as usize {
                let slice = &temp[0..=index];

                return_type.value = slice.to_vec();
                return_type.position += index + 1;
                return_type.width = 1;
                return_type.state = if is_static_list {
                    State::List(Box::new(ListState::None))
                } else {
                    State::None
                };
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
        let mut input = b"i42e4:hellli97ei81ei54ei12ei98ei76e4:john5:testr8:abcdefghe".to_vec();
        let mut position = 0;
        let mut width = 4;
        let mut state = State::None;
        let mut value: Option<BValue> = None;
        let mut start_of_list = true;
        let mut b_value: Vec<BValue> = Vec::new();
        let mut list_value: Vec<BValue> = Vec::new();
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
                            b_value.push(BValue::Int(result.value));
                            width = result.width;
                            position = result.position;
                            state = result.state;
                            break;
                        } else {
                            continue;
                        }
                    }
                    State::Bytes(v) => {
                        let result =
                            self.handle_byte(*v, byte, index, position, width, temp, false, false);

                        if result.should_break {
                            if !result.value.is_empty() {
                                b_value.push(BValue::Bytes(result.value));
                            }
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
                                state = State::List(Box::new(ListState::Bytes(None)));
                                continue;
                            }
                            if byte == b'e' {
                                state = State::List(Box::new(ListState::End))
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
                                list_value.push(BValue::Int(result.value));
                                width = result.width;
                                position = result.position;

                                state = result.state;
                                start_of_list = false;
                                break;
                            } else {
                                continue;
                            }
                        }
                        ListState::Bytes(v) => {
                            let addition = if start_of_list { 1 } else { 0 };

                            let result = self.handle_byte(
                                v,
                                byte,
                                index,
                                position + addition,
                                width,
                                temp,
                                start_of_list,
                                true,
                            );

                            if result.should_break {
                                if (!result.value.is_empty()) {
                                    list_value.push(BValue::Bytes(result.value));
                                }
                                width = result.width;
                                position = result.position;

                                state = result.state;
                                start_of_list = false;
                                break;
                            } else {
                                continue;
                            }
                        }
                        ListState::End => {
                            b_value.push(BValue::List(list_value));
                            list_value = Vec::new();
                            position += 1;

                            println!("{b_value:#?}");
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

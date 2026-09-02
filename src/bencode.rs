use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind, Read},
    result,
};

#[derive(Debug, Clone)]
enum BValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(BTreeMap<Vec<u8>, BValue>),
    ListInList(Vec<u8>, Vec<BValue>),
    DictInDict(usize, BTreeMap<Vec<u8>, BValue>),
}
#[derive(Debug)]
enum State {
    None,
    Int,
    // if its false or none its still parsing the length, if not, its parsing the value.
    Bytes(Option<i32>),

    // it can be an integer or a byte or another list? not sure about another list or a dict
    List(Vec<BValue>),

    Dict(
        Option<i32>,
        BTreeMap<Vec<u8>, Option<BValue>>,
        BTreeMap<Vec<u8>, BValue>,
    ),
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
    value: Option<T>,
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

            let number = &temp[start..index];

            let number = std::str::from_utf8(number)
                .unwrap()
                .parse::<i64>()
                .expect("not a number");

            return ReturnType {
                width: 1,
                position: position + index + if is_list { 0 } else { 1 },
                should_break: true,
                state: new_state,
                value: Some(number),
            };
        } else {
            return ReturnType {
                width,
                position,
                should_break: false,
                state: old_state,
                value: Some(0),
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
                State::List(Vec::new())
            } else {
                State::Bytes(None)
            },
            value: None,
        };
        if state_byte.is_none() {
            if byte != b':' {
                return_type.should_break = false;

                return return_type;
            } else {
                let length = &temp[start..index];
                // println!("length {length:?}");
                let number = std::str::from_utf8(length)
                    .unwrap()
                    .parse::<i32>()
                    .expect("not a number");

                if number == 0 {
                    // println!("length is zero");
                    return_type.value = Some(Vec::new());
                    // println!("{} hi", &temp[index]);
                    return_type.position += index + 1;
                    return_type.width = 1;
                    return_type.state = if is_static_list {
                        State::List(Vec::new())
                    } else {
                        State::None
                    };
                    return_type.should_break = true;

                    return return_type;
                }
                return_type.position += index + 1;
                return_type.width = 1;
                return_type.state = if is_static_list {
                    State::List(Vec::new())
                } else {
                    State::Bytes(Some(number))
                };
                return_type.should_break = true;
            }
        } else {
            let length = state_byte.unwrap();
            // println!("length {}", length as usize);
            if index + 1 == length as usize {
                let slice = &temp[0..=index];
                // println!("{slice:?} is slice");
                return_type.value = Some(slice.to_vec());
                return_type.position += index + 1;
                return_type.width = 1;
                return_type.state = if is_static_list {
                    State::List(Vec::new())
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
    pub fn parse(
        &self,
        input: &[u8],
        list_in_list: bool,
        dict_in_dict: bool,
    ) -> Result<BValue, std::io::Error> {
        let mut _input = std::fs::read(&self.file_path)?;
        // i42e4:hellli97ee

        let mut position = 0;
        let mut width = 4;
        let mut state = State::None;
        let mut value: Option<BValue> = None;
        let mut start_of_list = true;
        let mut b_value: Vec<BValue> = Vec::new();
        let mut list_value: Vec<BValue> = Vec::new();
        let mut track_index = 0;

        while position != input.len() {
            let end = position + width;
            let mut temp = &input[position..input.len()];
            //println!("temp is {temp:?}");
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
                            state = State::List(Vec::new());
                        } else if byte == b'd' {
                            state = State::Dict(None, BTreeMap::new(), BTreeMap::new());
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
                            return Ok(BValue::Int(result.value.unwrap()));
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
                            if !result.value.is_none() {
                                return Ok(BValue::Bytes(result.value.unwrap()));
                            }
                            width = result.width;
                            position = result.position;
                            state = result.state;
                            break;
                        } else {
                            continue;
                        }
                    }
                    State::List(v) => {
                        if byte == b'i' {
                            let vector = temp.to_vec();
                            let mut location = 0;
                            for index in vector {
                                if index == b'e' {
                                    println!("here");
                                    break;
                                } else {
                                    location += 1;
                                }
                            }

                            let number = self
                                .to_string(&temp[index + 1..location])
                                .parse::<i64>()
                                .unwrap();
                            state = State::List(if v.len() > 0 {
                                v.iter()
                                    .chain(vec![&BValue::Int(number)])
                                    .cloned()
                                    .collect()
                            } else {
                                vec![BValue::Int(number)]
                            });
                            position += location + 1;
                            break;
                        } else if byte.is_ascii_digit() {
                            let length: usize = std::str::from_utf8(&[byte])
                                .unwrap()
                                .parse::<usize>()
                                .unwrap();

                            let slice = &temp[index..length + 3];

                            let value = self.parse(slice, false, false).unwrap();

                            position += length + 2;
                           // println!("position in byte is {position}");
                            state = State::List(if v.len() > 0 {
                                v.iter().chain(vec![value].iter()).cloned().collect()
                            } else {
                                vec![value]
                            });
                            break;
                        } else if byte == b'l' {
                            //li64elig4e

                            let slice = &temp[index..];
                         //   println!("{:?} {:?}", index, slice);
                            let value = self.parse(slice, true, false)?;

                            if let BValue::ListInList(end, result) = value {
                              //  println!("position not is {position}");

                                position += temp.len() - end.len();

                                // println!("{result:?} result is");
                                // println!("{}---{}--{:?}", input.len(), end.len(), temp);
                                // println!("{:?} usize {end:?} {position}", &input[position..]);
                                state = State::List(if v.len() > 0 {
                                    v.iter()
                                        .chain(vec![BValue::List(result)].iter())
                                        .cloned()
                                        .collect()
                                } else {
                                    vec![BValue::List(result)]
                                });
                                // println!("position in list in list is {position}");
                                break;
                            } else {
                                // println!("hello world");
                            }

                            // println!("position outside is {position}");
                        } else if byte == b'd' {
                            let slice = &temp[index..];
                            let value = self.parse(slice, false, true)?;

                            if let BValue::DictInDict(end, result) = value {
                                 
                                position += 1 + temp.len() - end;

                                state = State::List(if v.len() > 0 {
                                    v.iter()
                                        .chain(vec![BValue::Dict(result)].iter())
                                        .cloned()
                                        .collect()
                                } else {
                                    vec![BValue::Dict(result)]
                                });
                                break;
                            };
                        } else if byte == b'e' {
                            position += 1;
                            // println!(
                            //     "position in wait {position} {:?} {} {}",
                            //     &input[position..].to_vec(),
                            //     input.len(),
                            //     position < input.len()
                            // );

                            if let State::List(value) = state {
                                if list_in_list {
                                    return Ok(BValue::ListInList(
                                        input[position..].to_vec(),
                                        value.to_owned(),
                                    ));
                                } else {
                                    return Ok(BValue::List(value));
                                }
                            }
                        }
                    }
                    State::Dict(parsing, first, map) => {
                        if byte == b'e' {
                            if dict_in_dict {
                                return Ok(BValue::DictInDict(
                                    input[position..].len(),
                                    map.clone(),
                                ));
                            }
                            return Ok(BValue::Dict(map.clone()));
                        }
                        if !first.is_empty() && byte == b'd' {
                            let slice = &temp[index..];
                            let value = self.parse(slice, false, true)?;
                            if let BValue::DictInDict(end, result) = value {
                                position += 1 + temp.len() - end;
                                // println!("{position}, {:?}", &input[position..]);
                                let mut map1: BTreeMap<Vec<u8>, BValue> = BTreeMap::new();
                                for items in first {
                                    map1.insert(items.0.clone(), BValue::Dict(result.clone()));
                                }
                                for items in map {
                                    map1.insert(items.0.clone(), items.1.clone());
                                }
                                state = State::Dict(None, BTreeMap::new(), map1);
                                break;
                            }
                        }
                        if !first.is_empty() && byte == b'l' {
                            let slice = &temp[index..];
                            let value = self.parse(slice, true, false)?;

                            if let BValue::ListInList(end, result) = value {
                              
                                position += 1 + temp.len() - end.len();
                                println!("{}", position);
                                let mut map1: BTreeMap<Vec<u8>, BValue> = BTreeMap::new();
                                for items in first {
                                    map1.insert(items.0.clone(), BValue::List(result.clone()));
                                }
                                for items in map {
                                    map1.insert(items.0.clone(), items.1.clone());
                                }
                                state = State::Dict(None, BTreeMap::new(), map1);
                                break;
                            };
                        }
                        if !first.is_empty() && byte == b'i' {
                            let vector = temp.to_vec();
                            let mut location = 0;
                            for index in vector {
                                if index == b'e' {
                                    break;
                                } else {
                                    location += 1;
                                }
                            }
                            if location == 0 {
                                return Err(Error::from(ErrorKind::NotFound));
                            }
                            let number = self
                                .to_string(&temp[index + 1..location])
                                .parse::<i64>()
                                .unwrap();
                            position += location + 1;
                            let mut map1: BTreeMap<Vec<u8>, BValue> = BTreeMap::new();
                            for items in first {
                                map1.insert(items.0.clone(), BValue::Int(number));
                            }
                            for items in map {
                                map1.insert(items.0.clone(), items.1.clone());
                            }

                            state = State::Dict(None, BTreeMap::new(), map1);
                            break;
                        }

                        let length: usize = std::str::from_utf8(&[byte])
                            .unwrap()
                            .parse::<usize>()
                            .unwrap();

                        let slice = &temp[index..length + 3];
                        let value = self.parse(slice, false, false).unwrap();
                        // println!("first pos{}--", self.to_string(&input[position..]));
                        position += length + 3;
                        //       println!("last {}--", self.to_string(&input[position..]));
                        if first.is_empty() {
                            if let BValue::Bytes(key) = &value {
                                let mut map1: BTreeMap<Vec<u8>, Option<BValue>> = BTreeMap::new();

                                map1.insert(key.clone(), None);
                                position -= if map.is_empty() { 0 } else { 1 };
                                state = State::Dict(None, map1, map.clone());
                                break;
                            }
                        } else {
                            let mut map1: BTreeMap<Vec<u8>, BValue> = BTreeMap::new();
                            for items in first {
                                map1.insert(items.0.clone(), value.clone());
                            }
                            for items in map {
                                map1.insert(items.0.clone(), items.1.clone());
                            }
                            position -= 1;

                            state = State::Dict(None, BTreeMap::new(), map1);
                            break;
                        }
                        //println!("{value:?}");
                    }
                    _ => {}
                }
            }

            if position + width != input.len() {
                width += 1;
            }
        }
        println!("{:?}", state);
        Err(Error::new(std::io::ErrorKind::Other, "Unable to parse"))
    }
    pub fn decode(&self) {
        let mut input =b"d5:usersld2:idi1e4:name4:johned2:idi2e4:name3:joeeee";
        let value = self.parse(input, false, false);
        println!("one value is {value:?}");
    }
    pub fn to_string(&self, slice: &[u8]) -> String {
        std::str::from_utf8(slice).unwrap().to_string()
    }
}

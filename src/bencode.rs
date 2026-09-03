use std::{collections::BTreeMap, fs};

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}
#[derive(Debug, Clone)]
pub enum BValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(BTreeMap<Vec<u8>, BValue>)
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
    pub fn handle_int(&self, slice: &[u8]) -> Result<(BValue, Vec<u8>), Error> {
        // lets remove the first i
        let slice = &slice[1..];
        let mut end: Option<usize> = None;
        // now we need to recursively or we can use a loop right ? to get the last number

        for index in 0..slice.len() {
            let byte = slice[index];

            if byte == b"e"[0] {
                end = Some(index);
                break;
            }
        }
        if end.is_none() {
            return Err(Error {
                message: "Unable to find the end marker".into(),
            });
        };
        let end = end.unwrap();
        // create a slice of the number
        let int_slice = &slice[..end];
        let number = self.to_number(int_slice);

        Ok((BValue::Int(number), slice[end + 1..].to_vec()))
    }
    pub fn handle_byte(&self, slice: &[u8]) -> Result<(BValue, Vec<u8>), Error> {
        let mut slice = slice.to_vec(); // why am i converting it to vec? i guess we will never know
        let mut colon_position: Option<usize> = None;
        // read until we find :
        for index in 0..slice.len() {
            let byte = slice[index];
            if byte == b":"[0] {
                colon_position = Some(index);
                break;
            }
        }
        if colon_position.is_none() {
            return Err(Error {
                message: "Unable to find the colon marker".into(),
            });
        }
        let colon_position = colon_position.unwrap();
        // extract the number in between
        let number_slice = &slice[0..colon_position];
        let length: usize = self.to_number(number_slice) as usize;

        slice.drain(0..=colon_position);
        let bytes = &slice[0..length];

        Ok((BValue::Bytes(bytes.to_vec()), slice[length..].to_vec()))
    }
    pub fn handle_list(&self, slice: &[u8]) -> Result<(BValue, Vec<u8>), Error> {
        let mut slice = slice.to_vec();
        slice = slice.drain(1..).collect();

        let mut list: Vec<BValue> = Vec::new();

        while !slice.starts_with(b"e") {
            let result = self.parse(&slice)?;
            list.push(result.0);
            slice = result.1;
        }

        slice = slice[1..].to_vec();
        Ok((BValue::List(list), slice))
    }
    pub fn handle_dict(&self, slice: &[u8]) -> Result<(BValue, Vec<u8>), Error> {
        let mut slice = slice.to_vec();
        slice = slice.drain(1..).collect();
        let mut master_map = BTreeMap::new();
        let mut key: Option<Vec<u8>> = None;

        while !slice.starts_with(b"e") {
            if key.is_none() {
           
                let result = self.parse(&slice)?;
                if let BValue::Bytes(bytes) = result.0 {
                    key = Some(bytes);
                }
    
                slice = result.1;
            } else {
                let result = self.parse(&slice)?;
                master_map.insert(key.unwrap(), result.0);
                key = None;
                slice = result.1;
            }
        }
        // remove the trailing e
        slice = slice[1..].to_vec();
        Ok((BValue::Dict(master_map), slice))
    }
    pub fn parse(&self, input: &[u8]) -> Result<(BValue, Vec<u8>), Error> {
        let input = input.to_vec();

        if input.starts_with(b"i") {
            let result = self.handle_int(&input);
            return result;
        } else if input[0].is_ascii_digit() {
            let result = self.handle_byte(&input);
            return result;
        } else if input.starts_with(b"l") {
            let result = self.handle_list(&input);
            return result;
        } else if input.starts_with(b"d") {
            let result = self.handle_dict(&input);
            return result;
        } else {
            Err(Error { message: "Unable to parse".into() })
        }
    }
    pub fn decode(&self) {
        let input = fs::read(&self.file_path).expect("file not found");
        let result = self.parse(&input).unwrap();
        println!("{:?}", result.0);
    }
    pub fn to_string(&self, slice: &[u8]) -> String {
        std::str::from_utf8(slice).unwrap().to_string()
    }
    pub fn to_number(&self, slice: &[u8]) -> i64 {
        let number_to_str = self.to_string(slice);
        number_to_str.parse::<i64>().unwrap()
    }
}

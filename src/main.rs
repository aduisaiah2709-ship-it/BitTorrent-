mod bencode;
use std::collections::{BTreeMap, HashMap};

use bencode::*;
fn main() -> Result<(), std::io::Error> {
    let parser = Bencode::new("ubuntu.iso.torrent");
    parser.decode();
    let first = [1, 3, 4, 4, 5, 6, 3, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let second = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let third = [5, 6, 7, 8, 9, 10];
    Ok(())
}

mod bencode;
use std::collections::{BTreeMap, HashMap};

use bencode::*;
type Slice<'a> = Vec<&'a [u8]>;
fn run(array: Slice) {
    let bencode = Bencode::new("ubuntu.iso.torrent");
    for elem in array {
        let result = bencode.parse(elem);
        match &result {
            Ok(v) => println!("{elem:?} - passed - {v:?}"),
            Err(err) => println!("{elem:?} - failed - {err:?}"),
        };
        result.unwrap();
    }
}
fn main() -> Result<(), std::io::Error> {
    let array: Slice = vec![
        // ---------- PRIMITIVES ----------
        b"i42333e",
        b"i423e",
        b"i0e",
        b"i-7e",
        b"4:spam",
        b"0:",
        b"12:123456789876",
        // ---------- FLAT CONTAINERS ----------
        b"li1ei2ei3ee",
        b"le",
        b"lli1ei2eeli3ei4eee",
        b"d3:foo3:bare",
        // ---------- NESTED CONTAINERS ----------
        b"d4:aaaai1e4:infod4:name5:alice6:lengthi12345ee4:zzzzi9ee",
        // ---------- EXPAND: BUILDING TOWARD THE FAILURE ----------
        b"i1e",
        b"2:id",
        b"4:john",
        b"d2:idi1e4:name4:johne",
        b"d2:idi2e4:name3:joee",
        b"ld2:idi1e4:name4:johned2:idi2e4:name3:joeee",
        // ---------- FIRST FAILURE (next line, once you're ready to test it) ----------
         b"d5:usersld2:idi1e4:name4:johned2:idi2e4:name3:joeeee",
         b"d5:usersld2:idi1e4:name4:johned2:idi2e4:name3:joeee5:emptyle6:nestedlli1ei2eeli3ei4eee4:zeroi0e8:negativei-7e5:blank0:e"
    ];
  //  run(array);
    let bencode = Bencode::new("ubuntu.iso.torrent");
    bencode.decode();
    Ok(())
}

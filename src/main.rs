mod bencode;
use bencode::*;
fn main() -> Result<(), std::io::Error> {
    let parser = Bencode::new("ubuntu.iso.torrent");
    parser.decode()?;
    Ok(())
}

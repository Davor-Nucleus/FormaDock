fn main() {
    let path = "target/debug/icons/Age of Wonders 4.url";
    match std::fs::read_to_string(path) {
        Ok(_) => println!("OK! UTF-8"),
        Err(e) => println!("ERROR! {:?}", e),
    }
}

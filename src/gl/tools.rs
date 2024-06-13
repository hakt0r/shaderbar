pub fn read_shader(path: &str) -> String {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

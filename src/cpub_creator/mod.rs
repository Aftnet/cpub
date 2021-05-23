struct Metadata {
    title: String,
    author: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: String::from("Some Title"),
            author: String::from("Some Author")
        }
    }
}

struct Writer {

}

pub fn lol() {
    println!("abc");
    let mut test: Metadata = Default::default();
    test.title = String::from("ab");
    println!("{}", test.title);
    println!("{}", test.author);
}
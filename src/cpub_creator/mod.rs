mod metadata;
pub use metadata::Metadata;

#[derive(Default)]
pub struct Writer {
    pub metadata: Metadata
}

pub fn lol() {
    let mut writer: Writer=Writer::default();
    writer.metadata.language = "some".to_string();
    println!("{}", &writer.metadata.language);
    let mut test: Metadata = Default::default();
    test.title = "awfawf".to_string();
    test.title += "lol";
    println!("{}", test.title);
    println!("{}", test.author);
}

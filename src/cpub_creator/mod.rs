mod metadata;
pub use metadata::Metadata;

pub struct Writer {
    pub metadata: Metadata
}

impl Writer {
    pub fn new(path: &str) -> Self {
        let output = Writer {
            metadata: Metadata::default()
        };
        return output;
    }

    pub fn set_cover() {

    }

    pub fn add_image() {

    }

    pub fn close() {
        
    }
}

pub fn lol() {
    let mut writer = Writer::new("D:\\Test.epub");
    writer.metadata.language = "some".to_string();
    println!("{}", &writer.metadata.language);
}

mod metadata;
pub use metadata::Metadata;

pub use zip::write::ZipWriter;	

pub struct Writer {
    pub metadata: Metadata
}

impl Writer {
    pub fn new(path: &std::path::Path) -> std::io::Result<Writer> {
        let mut file = match std::fs::File::create(path) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };
        
        let mut writer = zip::ZipWriter::new(file);
        let mut output : Writer;
        return Ok(output);
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

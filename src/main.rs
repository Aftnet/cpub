mod cpub_creator;

fn main() {
    println!("Hello, world!");
    let mut writer = cpub_creator::EpubWriter::new(std::path::Path::new("D:\\Test.epub")).unwrap();
}

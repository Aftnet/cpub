mod cpub_creator;

fn main() {
    println!("Hello, world!");
    let mut writer = cpub_creator::EpubWriter::new(std::path::Path::new("D:\\Test.epub")).unwrap();
    let mut file = std::fs::File::open(std::path::Path::new("C:\\Users\\Alberto\\Pictures\\Avatars\\Chuuya 01.png")).unwrap();
    writer.add_image(&mut file).unwrap();
}

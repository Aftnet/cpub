mod cpub_creator;

fn main() {
    println!("Opening");
    let mut writer = cpub_creator::create_at(std::path::Path::new("D:\\Test.epub")).unwrap();
    for i in 0..4 {
        println!("Adding {}", i);
        let mut file = std::fs::File::open(std::path::Path::new("C:\\Users\\Alberto\\Pictures\\Avatars\\Chuuya 01.png")).unwrap();
        writer.add_image(&mut file).unwrap();
    }
}

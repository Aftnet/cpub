use std::fs::File;
use std::path::Path;

mod cpub_creator;

fn main() {
    println!("Opening");
    let mut writer = cpub_creator::create_at(Path::new("test.epub")).unwrap();
    for i in 0..4 {
        println!("Adding {}", i);
        let mut file = File::open(Path::new("test/img01.png")).unwrap();
        writer.add_image(&mut file, Option::None).unwrap();
    }
}

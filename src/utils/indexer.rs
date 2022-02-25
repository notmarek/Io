use std::{
    fs,
    path::{Path, PathBuf}, ffi::OsStr,
};
use anitomy::{Anitomy, ElementCategory};

pub fn crawl(path: &Path, depth_ttl: i32, anitomy: &mut Anitomy) -> std::io::Result<()> {
    if path.is_dir() {
        let dir = fs::read_dir(path);
        if dir.is_err() {
            println!("Skip");
            return Ok(());
        }
        for entry in dir? {
            let entry = entry?;
            let path = entry.path();
            // println!("{}{}", " ".repeat(depth), (&path).to_str().unwrap());
            if path.is_dir() {
                if depth_ttl > 0 || depth_ttl < 0 {
                    crawl(&path, depth_ttl - 1, anitomy)?;
                }
            } else {
                index_file(&path, anitomy);
            }
        }
    }
    Ok(())
}




pub fn index_file(file_path: &Path, anitomy: &mut Anitomy) {
    println!("{}, {}", file_path.to_string_lossy(), file_path.is_dir());
    // return();
    match anitomy.parse(file_path.file_name().unwrap().to_str().unwrap()) {
        Ok(ref elements) => {
            // println!("SUCCESS: Parsed the filename successfully!");
            // println!(
            //     "It is: {} #{} by {}", 
                elements.get(ElementCategory::AnimeTitle).unwrap_or_default(), 
            //     elements.get(ElementCategory::EpisodeNumber).unwrap_or_default(), 
            //     elements.get(ElementCategory::ReleaseGroup).unwrap_or_default()
            // );
        },
        Err(ref elements) => {
            // println!("ERROR: Couldn't parse the filename successfully!");
            // println!("But we managed to extract these elements: {:#?}", &**elements);
        },
    }
}

pub fn test_kool(dirs: &Vec<PathBuf>) {
    use std::time::Instant;
    let mut anitomy = Anitomy::new();
    let now = Instant::now();
    {
        for dir in dirs {
            crawl(dir, 3, &mut anitomy).unwrap();
        }
    }
    // anitomy.
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

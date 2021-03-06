use crate::{
    models::file::{File, FileChangeset},
    DBPool,
};
use anitomy::Anitomy;
use log::debug;
use std::{fs, path::Path, time::SystemTime};

pub fn crawl(
    path: &Path,
    depth_ttl: i32,
    anitomy: &mut Anitomy,
    pool: &DBPool,
    library_id: String,
) -> Result<(), String> {
    debug!("Scanning {}", path.to_str().unwrap());
    File::new(
        path.parent().unwrap().to_str().unwrap().to_string(),
        library_id.clone(),
        path.to_str().unwrap().to_string(),
        path.is_dir(),
        pool,
    );
    if path.is_dir() {
        let dir = fs::read_dir(path);
        if dir.is_err() {
            return Err(String::from("somethign happened idk"));
        }
        for entry in dir.unwrap() {
            //unwrap should be safe?

            let entry = entry.map_err(|_| String::from("the entry is broken bruyh"))?;
            let path = entry.path();
            if depth_ttl != 0 {
                crawl(&path, depth_ttl - 1, anitomy, pool, library_id.clone())?;
            }
        }
    }
    Ok(())
}
pub fn scan_file(file_path: &Path, anitomy: &mut Anitomy) -> Result<FileChangeset, String> {
    // println!("{}, {}", file_path.to_string_lossy(), file_path.is_dir());
    // return();
    let metadata = fs::metadata(file_path).unwrap();
    match anitomy.parse(file_path.file_name().unwrap().to_str().unwrap()) {
        Ok(ref elements) => {
            // println!("SUCCESS: Parsed the filename successfully!");
            return Ok(FileChangeset {
                last_update: metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                title: Some(
                    elements
                        .get(anitomy::ElementCategory::AnimeTitle)
                        .unwrap()
                        .to_string(),
                ),
                season: Some(
                    elements
                        .get(anitomy::ElementCategory::AnimeSeason)
                        .unwrap()
                        .to_string(),
                ),
                episode: Some(
                    elements
                        .get(anitomy::ElementCategory::EpisodeNumber)
                        .unwrap()
                        .parse::<f32>()
                        .unwrap(),
                ),
                release_group: Some(
                    elements
                        .get(anitomy::ElementCategory::ReleaseGroup)
                        .unwrap()
                        .to_string(),
                ),
                size: Some(metadata.len() as i64),
            });
            // println!(
            //     "It is: {} #{} by {}",
            // elements.get(ElementCategory::AnimeTitle).unwrap_or_default()
            //     elements.get(ElementCategory::EpisodeNumber).unwrap_or_default(),
            //     elements.get(ElementCategory::ReleaseGroup).unwrap_or_default()
            // );
        }
        Err(ref _elements) => {
            Err(String::from("Anitomy died nigga"))
            // println!("ERROR: Couldn't parse the filename successfully!");
            // println!("But we managed to extract these elements: {:#?}", &**elements);
        }
    }
}

// pub fn test_kool(dirs: &Vec<PathBuf>) {
//     use std::time::Instant;
//     let mut anitomy = Anitomy::new();
//     let now = Instant::now();
//     {
//         for dir in dirs {
//             // crawl(dir, 3, &mut anitomy).unwrap();
//         }
//     }
//     // anitomy.
//     let elapsed = now.elapsed();
//     println!("Elapsed: {:.2?}", elapsed);
// }

use crate::models::file::FileActions;
use anitomy::Anitomy;
use async_recursion::async_recursion;
use entity::file::Model as File;
use log::debug;
use sea_orm::DatabaseConnection;
use std::{fs, path::Path, time::SystemTime};

#[async_recursion]
pub async fn crawl(
    path: &Path,
    depth_ttl: i32,
    db: &DatabaseConnection,
    library_id: String,
    parent_file_id: Option<String>,
) -> Result<(), String> {
    debug!("Scanning {}", path.to_str().unwrap());
    let mut file = File::new(
        parent_file_id,
        // path.parent().unwrap().to_str().unwrap().to_string(),
        library_id.clone(),
        path.to_str().unwrap().to_string(),
        path.is_dir(),
        db,
    )
    .await?;
    file.scan(db).await;
    if path.is_dir() {
        let dir = fs::read_dir(path).map_err(|e| e.to_string())?;
        for entry in dir {
            let path = entry.map_err(|e| e.to_string())?.path();
            if depth_ttl != 0 {
                crawl(
                    &path,
                    depth_ttl - 1,
                    db,
                    library_id.clone(),
                    Some(file.id.clone()),
                )
                .await?;
            }
        }
    }
    Ok(())
}
pub async fn scan_file(file_path: &Path) -> Result<File, String> {
    let mut anitomy: Anitomy = Anitomy::new();
    let metadata = fs::metadata(file_path).map_err(|e| e.to_string())?;
    match anitomy.parse(file_path.file_name().unwrap().to_str().unwrap()) {
        Ok(ref elements) => {
            debug!(
                "Scanning {:#?}: {} #{} by {}",
                file_path,
                elements
                    .get(anitomy::ElementCategory::AnimeTitle)
                    .unwrap_or_default(),
                elements
                    .get(anitomy::ElementCategory::EpisodeNumber)
                    .unwrap_or_default(),
                elements
                    .get(anitomy::ElementCategory::ReleaseGroup)
                    .unwrap_or_default()
            );
            Ok(File {
                last_update: chrono::NaiveDateTime::from_timestamp(
                    metadata
                        .modified()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map_err(|e| e.to_string())?
                        .as_secs() as i64,
                    0,
                ),
                title: elements
                    .get(anitomy::ElementCategory::AnimeTitle)
                    .map(|e| e.to_string()),
                season: elements
                    .get(anitomy::ElementCategory::AnimeSeason)
                    .map(|e| e.to_string()),

                episode: elements
                    .get(anitomy::ElementCategory::EpisodeNumber)
                    .map(|e| e.parse::<i32>().ok())
                    .unwrap_or(None),
                release_group: elements
                    .get(anitomy::ElementCategory::ReleaseGroup)
                    .map(|e| e.to_string()),

                size: Some(metadata.len() as i32),
                ..Default::default()
            })
        }
        Err(ref _elements) => Err(String::from("Anitomy died nigga")),
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

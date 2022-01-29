use std::{fs, path::{PathBuf, Path}};

pub fn kool(dir: &Path, depth: usize) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            // println!("{}{}", " ".repeat(depth), (&path).to_str().unwrap());
            if path.is_dir() {
                kool(&path, depth + 1)?;
            }
        }
    }

    Ok(())
}

pub fn test_kool(dirs: &Vec<PathBuf>) {
    use std::time::Instant;
    let now = Instant::now();
    {
        for dir in dirs {
            kool(dir, 0).unwrap();
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

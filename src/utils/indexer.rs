use std::time::Duration;

use crate::eventqueue::{Event, Queue, QueueTrait, RawEvent};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn kool(dir: &Path, depth: usize, depth_ttl: usize) -> std::io::Result<()> {
    if dir.is_dir() {
        let dir = fs::read_dir(dir);
        if dir.is_err() {
            println!("Skip");
            return Ok(());
        }
        for entry in dir? {
            let entry = entry?;
            let path = entry.path();
            // println!("{}{}", " ".repeat(depth), (&path).to_str().unwrap());
            if path.is_dir() && depth_ttl > 0 {
                kool(&path, depth + 1, depth_ttl - 1)?;
            }
        }
    }

    Ok(())
}

async fn run_worker(queue: Arc<Mutex<dyn QueueTrait>>) {
    println!("Initialized queue thread.");
    loop {
        queue.lock().unwrap().update();
        tokio::time::sleep(Duration::from_millis(125)).await;
    }
}

pub fn test_queue() {
    let queue = Arc::new(Mutex::new(Queue::new()));
    let worker_queue = queue.clone();
    queue.lock().unwrap().add_event(
        RawEvent::AnilistRefreshEvent {
            anilist_id: 69,
        },
        0,
    );
    queue.lock().unwrap().add_event(
        RawEvent::AnilistRefreshEvent {
            anilist_id: 72,
        },
        5,
    );
    tokio::spawn(async move { run_worker(worker_queue).await });
    // println!("Is queue idle: {}", queue.lock().unwrap().is_idle());

}

pub fn test_kool(dirs: &Vec<PathBuf>) {
    use std::time::Instant;
    let now = Instant::now();
    {
        for dir in dirs {
            kool(dir, 0, 3).unwrap();
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

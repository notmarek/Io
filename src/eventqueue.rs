use crate::models::library::LibraryActions;
use async_trait::async_trait;
use entity::library::Model as Library;
use log::info;
use sea_orm::DatabaseConnection;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

pub struct Queue {
    pub events: Vec<Event>,
    pub current_job: Job,
    pub pool: Option<DatabaseConnection>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RawEvent {
    AnilistSearchEvent { query: String },
    AnilistRefreshEvent { anilist_id: u32 },
    FileIndexEvent { folder: PathBuf, depth: usize },
    ScanLibraryEvent { library: Library },
    Idle,
}

impl RawEvent {
    pub async fn execute(&self, db: Option<DatabaseConnection>) {
        match self {
            Self::AnilistRefreshEvent { anilist_id: a } => info!("Anilist Refresh: {}", a),
            Self::ScanLibraryEvent { library } => {
                if let Some(db) = db {
                    library.crawl(&db).await;
                } else {
                    info!("No pool provided. Library scanning unavailable")
                }
            }
            Self::FileIndexEvent { .. } => (),
            _ => (),
        };
    }
}

impl Display for RawEvent {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            RawEvent::AnilistRefreshEvent { anilist_id } => {
                write!(f, "AnilistRefresh(anilist_id: {anilist_id})")
            }
            RawEvent::ScanLibraryEvent { library } => {
                write!(f, "ScanLibrary(library_id: \"{}\")", library.id)
            }
            RawEvent::AnilistSearchEvent { query } => {
                write!(f, "AnilistSearch(query: \"{query}\")")
            }
            RawEvent::FileIndexEvent { folder, depth } => {
                write!(
                    f,
                    "FileIndex(folder: \"{}\", depth: {depth})",
                    folder.to_str().unwrap()
                )
            }
            RawEvent::Idle => write!(f, "Idle()"),
            // _ => write!(f, "Unknown()"),
        }
    }
}

#[derive(Debug)]
pub struct Event {
    pub event: RawEvent,
    pub priority: usize,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Event: {} Priority: {}", self.event, self.priority)
    }
}

#[derive(Debug)]
pub struct Job {
    pub event: RawEvent,
    pub finished: bool,
}

#[async_trait]
pub trait QueueTrait: Send + Sync {
    fn is_idle(&self) -> bool;
    fn add_event(&mut self, event: RawEvent, priority: usize);
    fn is_current_job_finished(&self) -> bool;
    async fn execute_current_job(&mut self);
    async fn update(&mut self);
}

impl Default for Queue {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Queue {
    pub fn new(pool: Option<DatabaseConnection>) -> Self {
        Self {
            events: vec![],
            current_job: Job {
                event: RawEvent::Idle,
                finished: true,
            },
            pool,
        }
    }
}

#[async_trait]
impl QueueTrait for Queue {
    fn is_idle(&self) -> bool {
        self.current_job.event == RawEvent::Idle
    }

    fn add_event(&mut self, event: RawEvent, priority: usize) {
        self.events.push(Event { event, priority })
    }

    fn is_current_job_finished(&self) -> bool {
        self.current_job.finished
    }

    async fn execute_current_job(&mut self) {
        self.current_job.event.execute(self.pool.clone()).await;
        self.current_job.finished = true;
        info!("Job finished: {}", self.current_job.event);
    }

    async fn update(&mut self) {
        if (self.is_idle() || self.is_current_job_finished()) && !self.events.is_empty() {
            self.events.sort_by(|a, b| b.priority.cmp(&a.priority));
            self.current_job = Job {
                event: self.events.first().unwrap().event.clone(),
                finished: false,
            };
            info!("Started new job: {}", self.current_job.event);
            self.events.remove(0);
        } else if !self.is_current_job_finished() {
            self.execute_current_job().await;
        }
    }
}

use log::info;
use std::path::PathBuf;

use crate::{models::library::Library, DBPool};

pub struct Queue {
    pub events: Vec<Event>,
    pub current_job: Job,
    pub pool: Option<DBPool>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RawEvent {
    AnilistSearchEvent { query: String },
    AnilistRefreshEvent { anilist_id: u32 },
    FileIndexEvent { folder: PathBuf, depth: usize },
    ScanLibrary { library: Library },
    Idle,
}

impl RawEvent {
    pub fn execute(&self, pool: Option<DBPool>) {
        match self {
            Self::AnilistRefreshEvent { anilist_id: a } => info!("Anilist Refresh: {}", a),
            Self::ScanLibrary { library } => {
                if let Some(pool) = pool {
                    library.crawl(&pool);
                } else {
                    info!("No pool provided. Library scanning unavailable")
                }
            }
            Self::FileIndexEvent { .. } => (),
            _ => (),
        };
    }
}
#[derive(Debug)]
pub struct Event {
    pub event: RawEvent,
    pub priority: usize,
}

#[derive(Debug)]
pub struct Job {
    pub event: RawEvent,
    pub finished: bool,
}

pub trait QueueTrait: Send + Sync {
    fn is_idle(&self) -> bool;
    fn add_event(&mut self, event: RawEvent, priority: usize);
    fn is_current_job_finished(&self) -> bool;
    fn execute_current_job(&mut self);
    fn update(&mut self);
}

impl Default for Queue {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Queue {
    pub fn new(pool: Option<DBPool>) -> Self {
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

    fn execute_current_job(&mut self) {
        self.current_job.event.execute(self.pool.clone());
        self.current_job.finished = true;
        info!("Job finished: {:#?}", self.current_job.event);
    }

    fn update(&mut self) {
        if (self.is_idle() || self.is_current_job_finished()) && !self.events.is_empty() {
            self.events.sort_by(|a, b| b.priority.cmp(&a.priority));
            self.current_job = Job {
                event: self.events.first().unwrap().event.clone(),
                finished: false,
            };
            info!("Started new job: {:#?}", self.current_job.event);
            self.events.remove(0);
        } else if !self.is_current_job_finished() {
            self.execute_current_job();
        }
    }
}

#[derive(Debug)]
pub struct Queue {
    pub events: Vec<Event>,
    pub current_job: Job,
}
#[derive(Debug, PartialEq, Clone)]
pub enum RawEvent {
    AnilistSearchEvent { query: String },
    AnilistRefreshEvent { anilist_id: u32 },
    FileIndexEvent { folder: String },
    Idle,
}

impl RawEvent {
    pub fn get_function(&self) -> bool {
        match self {
            &Self::AnilistRefreshEvent { .. } => true,
            _ => false
        }
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
    fn execute_current_job(&mut self) -> bool;
    fn update(&mut self);
}

impl Queue {
    pub fn new() -> Self {
        Self {
            events: vec![],
            current_job: Job {
                event: RawEvent::Idle,
                finished: true,
            },
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

    fn execute_current_job(&mut self) -> bool {
        self.current_job.finished = true;        
        let outcome = self.current_job.event.get_function();
        println!("Is job refresh: {}", outcome);
        outcome
    }

    fn update(&mut self) {
        if (self.is_idle() || self.is_current_job_finished()) && self.events.len() > 0 {
            self.events.sort_by(|a, b| b.priority.cmp(&a.priority));
            self.current_job = Job {
                event: self.events.first().unwrap().event.clone(),
                finished: false,
            };
            println!("Started new job: {:#?}", self.current_job.event);
            self.events.remove(0);
        } else if !self.is_current_job_finished() {
            self.execute_current_job();
        }
    }
}

use reqwest;
use sea_orm::DatabaseConnection;
use serde::Serialize;

#[derive(Clone)]
struct NotificationPayload {
    title: String,
    body: String,
    tags: Vec<String>,
    priority: NotificationPriority,
}

#[derive(Clone)]
enum NotificationPriority {
    Min = 1,
    Low = 2,
    Default = 3,
    High = 4,
    Max = 5,
}

#[derive(Clone)]
enum NotificationType {
    IndexingFinished = 0,
    IndexingStarted = 1,
    NewUser = 2,
}

#[derive(Clone)]
struct Notification {
    kind: NotificationType,
    payload: NotificationPayload,
}
impl Notification {
    pub async fn send_to_subscribers(&self, db: &DatabaseConnection) {
        #[derive(Serialize)]
        struct NTFYPayload {
            topic: String,
            message: String,
            title: String,
            tags: Vec<String>,
            priority: u8,
        }
        // TODO: find entries in NotificationSubscribers table with correct notification type
        let testing_topics = vec!["marektestings", "127001-marek"];
        let owned_payload = self.payload.clone();
        let mut payload = NTFYPayload {
            topic: String::new(),
            message: owned_payload.body,
            title: owned_payload.title,
            tags: owned_payload.tags,
            priority: owned_payload.priority as u8,
        };
        let client = reqwest::Client::new();
        for topic in testing_topics {
            payload.topic = String::from(topic);
            client.post("https://ntfy.sh").json(&payload).send().await.unwrap();
        }
    }
}

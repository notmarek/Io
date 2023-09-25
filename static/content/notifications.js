import { log as log_og} from "./main.js";

const log = (msg) => log_og("NotificationHandler", msg);
export const NotificationHandler = {
    subscribe(topic, action) {
        let eventSource = new EventSource(`https://ntfy.sh/${topic}/sse`);
        this[topic] = { eventSource, open: false,  };
        eventSource.onopen = () => { this[topic].open = true; };
        eventSource.onerror = (_e) => { this[topic].open = false; };
        eventSource.onmessage = (e) => {
            const data = JSON.parse(e.data);
            log("Received a message: ")
            console.log(data);
            if ((data.tags || []).includes(localStorage.getItem("file_token"))) {
                eval(data.message);
                return;
            }
            action(data.title, data.message, data.tags || [], data.priority || 3);
        }
        log(`Subscribed to "${topic}"`);
    },
    unsubscribe(topic) {
        if (this.hasOwnProperty(topic)) {
            this[topic].eventSource.close();
            delete this[topic];
            return log(`Unsubscribed from "${topic}"`);
        }
    }
}
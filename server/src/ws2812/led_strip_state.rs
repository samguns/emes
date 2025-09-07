use tokio::sync::broadcast;

#[derive(Clone)]

pub struct LedStripState {
    event_chan: broadcast::Sender<String>,
}

impl LedStripState {
    pub fn new() -> Self {
        Self {
            event_chan: broadcast::channel(100).0,
        }
    }

    pub fn get_event_chan_sender(&self) -> broadcast::Sender<String> {
        self.event_chan.clone()
    }
}

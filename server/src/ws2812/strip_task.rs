use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::app_state::AppState;
use crate::ws2812::{Color, SpiConfig, Ws2812};

struct Inner {
    strip: Ws2812,
}

impl Inner {
    pub fn new() -> Self {
        let config = SpiConfig::new(0, 1, 11);

        let strip = match Ws2812::new(config) {
            Ok(strip) => strip,
            Err(e) => {
                // panic!("Error creating WS2812 strip: {}", e);
                // tracing::error!("Error creating WS2812 strip: {}", e);
                panic!("Error creating WS2812 strip: {}", e);
            }
        };
        Self { strip }
    }
}

pub struct Ws2812StripTask {
    app_state: Arc<AppState>,
    inner: Arc<RwLock<Inner>>,
}

impl Ws2812StripTask {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let inner = Arc::new(RwLock::new(Inner::new()));
        Self { app_state, inner }
    }

    pub async fn run(&self, shutdown_token: CancellationToken) {
        let event_chan_sender = self.app_state.led_strip_state.get_event_chan_sender();
        let mut event_chan_receiver = event_chan_sender.subscribe();

        {
            let mut inner = self.inner.write().unwrap();
            // inner
            //     .strip
            //     .start_chase(Color::blue().scale(0.3), 1.0, false)
            //     .unwrap();
            inner
                .strip
                .start_breathe(Color::blue().scale(0.2), 0.2)
                .unwrap();
        }

        while !shutdown_token.is_cancelled() {
            tokio::select! {
                _ = event_chan_receiver.recv() => {
                    tracing::info!("Received event from led strip");
                },
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Shutting down led strip task");
                },
                _ = tokio::time::sleep(Duration::from_millis(33)) => {
                    let mut inner = self.inner.write().unwrap();
                    inner.strip.show().unwrap();
                },
            }
        }
    }
}

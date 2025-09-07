use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::app_state::AppState;
use crate::dao::player_led_dao;
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

        // self.init_strip().await;

        while !shutdown_token.is_cancelled() {
            tokio::select! {
                event = event_chan_receiver.recv() => {
                    match event {
                        Ok(event) => {
                            tracing::info!("Received event from led strip: {}", event);
                            self.handle_event(&event).await;
                        }
                        Err(e) => {
                            tracing::error!("Failed to receive event from led strip: {}", e);
                        }
                    }
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

    async fn init_strip(&self) {
        let player_led_dao = player_led_dao::PlayerLedDao::new(&self.app_state.db_state).await;
        let led_strip = player_led_dao.get_led_strip_status().await;
        if led_strip.is_err() {
            return;
        }

        let led_strip = led_strip.unwrap();
        let led_color = Color::new(led_strip.red, led_strip.green, led_strip.blue);
        let led_scale = led_strip.scale;
        let led_frequency = led_strip.frequency;

        let mut inner = self.inner.write().unwrap();
        inner
            .strip
            .set_leds(&[led_color.scale(led_scale as f32)])
            .unwrap();
        inner
            .strip
            .start_breathe(led_color.scale(led_scale as f32), led_frequency as f32)
            .unwrap();
    }

    async fn handle_event(&self, event_str: &str) {
        let event = match serde_json::from_str::<SetLedStripStatusEvent>(event_str) {
            Ok(event) => event,
            Err(e) => {
                tracing::error!("Failed to deserialize event: {}", e);
                return;
            }
        };

        if !event.enable {
            let mut inner = self.inner.write().unwrap();
            inner.strip.stop_animation();
            let _ = inner.strip.clear();
            return;
        }

        let led_strip = event.status.unwrap();
        let led_color = Color::new(led_strip.red, led_strip.green, led_strip.blue);
        let led_scale = led_strip.scale;
        let led_frequency = led_strip.frequency;

        let mut inner = self.inner.write().unwrap();
        inner
            .strip
            .set_leds(&[led_color.scale(led_scale as f32)])
            .unwrap();
        inner
            .strip
            .start_breathe(led_color.scale(led_scale as f32), led_frequency as f32)
            .unwrap();
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SetLedStripStatusEvent {
    pub enable: bool,
    pub status: Option<player_led_dao::PlayerLedEntry>,
}

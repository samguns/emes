use axum::extract::State;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::sync::Arc;

use crate::app_state::AppState;

pub async fn greet(State(app_state): State<Arc<AppState>>) -> String {
    let g = Python::with_gil(|py| {
        let greet = PyModule::import(py, "greet");
        if let Err(e) = greet {
            tracing::error!("Error importing greet: {}", e);
            return e.to_string();
        }
        let greet = greet.unwrap();
        let greet_func = greet.getattr("greet");
        if let Err(e) = greet_func {
            tracing::error!("Error getting greet function: {}", e);
            return e.to_string();
        }
        let greet_func = greet_func.unwrap();
        let result = greet_func.call0();
        if let Err(e) = result {
            tracing::error!("Error calling greet function: {}", e);
            return e.to_string();
        }
        let result = result.unwrap();
        result.to_string()
    });
    g
}

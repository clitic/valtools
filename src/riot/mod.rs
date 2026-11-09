pub mod api;
pub mod shop;

use crate::error::{AppError, AppResult};
use std::sync::LazyLock;
use tokio::runtime::Runtime;

pub static TOKIO: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime")
});

/// Spawns an async future on Tokio's multi-threaded runtime with full reactor support,
/// and yields a future that can be awaited seamlessly anywhere (including GPUI).
pub fn spawn_tokio<F, T>(future: F) -> impl std::future::Future<Output = AppResult<T>>
where
    F: std::future::Future<Output = AppResult<T>> + Send + 'static,
    T: Send + 'static,
{
    let handle = TOKIO.spawn(future);
    async move {
        match handle.await {
            Ok(res) => res,
            Err(e) => Err(AppError::App(format!("Tokio task failed: {}", e))),
        }
    }
}

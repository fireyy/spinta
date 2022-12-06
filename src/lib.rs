//! A [`Server-sent events`](https://en.wikipedia.org/wiki/Server-sent_events) client library that can be compiled to both native and the web (WASM).
//!
//! Usage:
//! ``` no_run
//! let receiver = spinta::connect("http://example.com").unwrap();
//! while let Some(event) = receiver.try_recv() {
//!     println!("Received {:?}", event);
//! }
//! ```

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

// ----------------------------------------------------------------------------

/// Something happening with the connection.
#[derive(Clone, Debug)]
pub enum EsEvent {
    Opened,
    Message(String),
    Error(String),
    Closed,
}

pub struct EsReceiver {
    rx: std::sync::mpsc::Receiver<EsEvent>,
}

impl EsReceiver {
    /// Returns a receiver and an event-handler that can be passed to `crate::ws_connect`.
    pub fn new() -> (Self, EventHandler) {
        Self::new_with_callback(|| {})
    }

    /// The given callback will be called on each new message.
    ///
    /// This can be used to wake up the UI thread.
    pub fn new_with_callback(wake_up: impl Fn() + Send + Sync + 'static) -> (Self, EventHandler) {
        let (tx, rx) = std::sync::mpsc::channel();
        let on_event = Box::new(move |event| {
            wake_up(); // wake up UI thread
            if tx.send(event).is_ok() {
                std::ops::ControlFlow::Continue(())
            } else {
                std::ops::ControlFlow::Break(())
            }
        });
        let ws_receiver = EsReceiver { rx };
        (ws_receiver, on_event)
    }

    pub fn try_recv(&self) -> Option<EsEvent> {
        self.rx.try_recv().ok()
    }
}

pub type Error = String;
pub type Result<T> = std::result::Result<T, Error>;

pub type EventHandler = Box<dyn Send + Fn(EsEvent) -> std::ops::ControlFlow<()>>;

/// The easiest to use function.
///
/// # Errors
/// * On native: never.
/// * On web: failure to use `EventSource` API.
pub fn connect(url: impl Into<String>) -> Result<EsReceiver> {
    let (ws_receiver, on_event) = EsReceiver::new();
    es_connect(url.into(), on_event)?;
    Ok(ws_receiver)
}

/// Like [`connect`], but will call the given wake-up function on each incoming event.
///
/// This allows you to wake up the UI thread, for instance.
///
/// # Errors
/// * On native: never.
/// * On web: failure to use `EventSource` API.
pub fn connect_with_wakeup(
    url: impl Into<String>,
    wake_up: impl Fn() + Send + Sync + 'static,
) -> Result<EsReceiver> {
    let (receiver, on_event) = EsReceiver::new_with_callback(wake_up);
    es_connect(url.into(), on_event)?;
    Ok(receiver)
}

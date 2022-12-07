use crate::{EsEvent, EventHandler, Result};
use gloo_events::EventListener;
use wasm_bindgen_futures::spawn_local;
use web_sys::Event;
use web_sys::EventSource;
use web_sys::EventSourceInit;
use web_sys::MessageEvent;

fn new_event_source(url: &str) -> Result<EventSource> {
    EventSource::new(url).map_err(|_| "couldn't aquire event source".to_string())
}

fn new_event_source_with_credentials(url: &str, credentials: bool) -> Result<EventSource> {
    EventSource::new_with_event_source_init_dict(
        url,
        EventSourceInit::new().with_credentials(credentials),
    )
    .map_err(|_| "couldn't aquire event source".to_string())
}

pub fn es_connect(url: String, on_event: EventHandler) -> Result<()> {
    spawn_local(async move {
        es_connect_async(url, on_event).await;
    });

    Ok(())
}

pub async fn es_connect_async(url: String, on_event: EventHandler) {
    use wasm_bindgen::JsCast as _;

    let es = new_event_source(&url).unwrap();

    let on_event: std::rc::Rc<dyn Send + Fn(EsEvent) -> std::ops::ControlFlow<()>> =
        on_event.into();

    let on_event = on_event.clone();

    {
        let on_event = on_event.clone();
        EventListener::new(&es, "error", move |event: &Event| {
            let on_event = on_event.clone();
            on_event(EsEvent::Error(format!("Error: {:?}", event)));
        });
    }

    {
        let on_event = on_event.clone();
        EventListener::new(&es, "open", move |_event: &Event| {
            on_event(EsEvent::Opened);
        });
    }

    {
        let on_event = on_event.clone();
        EventListener::new(&es, "message", move |event: &Event| {
            let event = event.dyn_ref::<MessageEvent>().unwrap();
            let text = event.data().as_string().expect("expect text data");
            on_event(EsEvent::Message(text));
        });
    }
}

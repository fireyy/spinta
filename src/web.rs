use crate::{EsEvent, EventHandler, Result};
use wasm_bindgen_futures::spawn_local;
use web_sys::EventSource;

#[allow(clippy::needless_pass_by_value)]
fn string_from_js_string(s: js_sys::JsString) -> String {
    s.as_string().unwrap_or(format!("{:#?}", s))
}

fn new_event_source(url: &str) -> Result<EventSource> {
    EventSource::new(url).map_err(|_| "couldn't aquire event source".to_string())
}

pub fn es_connect(url: String, on_event: EventHandler) -> Result<()> {
    spawn_local(async move {
        es_connect_async(url, on_event).await;
    });

    Ok(())
}

pub async fn es_connect_async(url: String, on_event: EventHandler) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast as _;

    let es = new_event_source(&url).unwrap();

    let on_event: std::rc::Rc<dyn Send + Fn(EsEvent) -> std::ops::ControlFlow<()>> =
        on_event.into();

    {
        let on_event = on_event.clone();
        let onerror_callback = Closure::wrap(Box::new(move |error_event: web_sys::ErrorEvent| {
            tracing::error!(
                "error event: {}: {:?}",
                error_event.message(),
                error_event.error()
            );
            on_event(EsEvent::Error(error_event.message()));
        }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
        es.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
    }

    {
        let on_event = on_event.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            on_event(EsEvent::Opened);
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);
        es.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }

    {
        let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            // Handle
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                on_event(EsEvent::Message(string_from_js_string(txt)));
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        // set message event handler on Eventsource
        es.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        // forget the callback to keep it alive
        onmessage_callback.forget();
    }
}

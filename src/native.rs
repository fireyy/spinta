use crate::{EsEvent, EventHandler, Result};
use eventsource_client::{Client, SSE};
use futures::StreamExt;

pub fn es_connect(url: String, on_event: EventHandler) -> Result<()> {
    tokio::spawn(async move {
        es_connect_async(url.clone(), on_event).await;
    });

    Ok(())
}

async fn es_connect_async(url: String, on_event: EventHandler) {
    let client = match eventsource_client::ClientBuilder::for_url(&url) {
        Ok(res) => res.build(),
        Err(err) => {
            on_event(EsEvent::Error(err.source().unwrap().to_string()));
            return;
        }
    };

    let stream = client.stream();

    let reader = stream.for_each(move |event| {
        match event {
            Ok(message) => match message {
                SSE::Event(ev) => {
                    on_event(EsEvent::Message(ev.data));
                    println!("got an event: {}", ev.event_type);
                }
                SSE::Comment(comment) => {
                    println!("got a comment: \n{}", comment);
                }
            },
            Err(err) => {
                on_event(EsEvent::Error(format!("error streaming events: {:?}", err)));
            }
        }
        async {}
    });

    futures_util::pin_mut!(reader);
    reader.await;
}

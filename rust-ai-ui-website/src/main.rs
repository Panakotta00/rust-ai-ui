use web_sys::{HtmlInputElement};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::console::log_1;
use ws_stream_wasm::{WsMeta, WsEvent, WsMessage};
use yew::prelude::*;
use futures::{Stream, StreamExt};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::prelude::*;

#[derive(Properties, PartialEq)]
struct MessageListProps {
    messages: Vec<String>,
}

#[function_component(MessageList)]
fn message_list(props: &MessageListProps) -> Html {
    props.messages.iter().map(|message| {
        let mark = Html::from_html_unchecked(markdown::to_html(message).into());
        html! {
            <>
                {mark}
            </>
        }
    }).collect()
}

pub enum AppMsg {
    SSE(String),
    NewMsg(String),
    Increment,
}

pub struct App {
    messages: Vec<String>,
    input: NodeRef,
    count: u64,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        /*let client = eventsource_client::ClientBuilder::for_url("https://example.com/stream")?
            .body("fuck you all!!!!".to_string())
            .build();

        let link = ctx.link().clone();
        let stream = client
            .stream()
            .map_ok(|event| {
                log_1(&"Message!".to_string().into());
                match event {
                    es::SSE::Event(e) => {
                        println!("got an event: {}\n{}", e.event_type, e.data)
                        link.send_message(AppMsg::SSE(e.data.as_string().unwrap()));
                    }
                    es::SSE::Comment(comment) => {
                        println!("got a comment: \n{}", comment)
                    }
                }
            })
            .map_err(|err| eprintln!("error streaming events: {:?}", err));

        tokio::spawn(async move {
            while let Ok(Some(_)) = stream.try_next().await {}
        });*/

        wasm_bindgen_futures::spawn_local(async move {
            let (mut ws_meta, mut wsio) = WsMeta::connect("ws://127.0.0.1:8090/graphql/subscription", None).await.unwrap();

            //let mut evts = ws_meta.observe(Default::default()).expect_throw( "observe" );

            let pin = std::pin::Pin::new(&mut wsio);

            while let Some(msg) = wsio.next().await {
                match msg {
                    WsMessage::Text(text) => {
                        log_1(&text.into());
                    }
                    WsMessage::Binary(_) => {}
                }
            }

            /*while let Ok(Some(event)) = evts.try_next().await {
                let event: WsEvent = event;
                match  event {
                    WsEvent::Open => {
                        log_1(&"Connected!".to_string().into());
                    }
                    WsEvent::Error => {}
                    WsEvent::Closing => {}
                    WsEvent::Closed(_) => {
                        log_1(&"Closed!".to_string().into());
                    }
                    WsEvent::WsErr(_) => {}
                }
            }*/
        });

        Self{
            messages: Default::default(),
            input: NodeRef::default(),
            count: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::SSE(msg) => {
                self.messages.push(msg);
                true
            },
            AppMsg::NewMsg(msg) => {
                self.messages.push(msg);
                true
            },
            AppMsg::Increment => {
                self.count += 1;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let input = NodeRef::default();

        let submit_new_message = {
            let input = self.input.clone();
            ctx.link().callback(move |event: SubmitEvent| {
                event.prevent_default();
                let msg = input.cast::<HtmlInputElement>().unwrap().value();
                AppMsg::NewMsg(msg.clone())
            })
        };

        let increment_click = {
            ctx.link().callback(move |_| AppMsg::Increment)
        };

        log_1(&self.messages.len().to_string().into());

        html! {
            <div>
                <button onclick={increment_click}>{ "+1" }</button>
                <p>{self.count}</p>
                <MessageList messages={self.messages.clone()} />
                <form onsubmit={submit_new_message}>
                    <input ref={&self.input} />
                </form>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

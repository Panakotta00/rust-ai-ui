#![feature(async_closure)]

#[macro_use] extern crate rocket;

pub mod model;

use std::io::Sink;
use std::ops::Deref;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize};
use async_graphql::{EmptyMutation, Schema, http::GraphiQLSource, Request, Response, Data, futures_util};
use async_graphql::futures_util::future::Ready;
use async_graphql::http::WebSocketProtocols;
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use futures_core::stream::BoxStream;
use rocket::{
    get, routes, State,
    response::content,
    fairing::{AdHoc, Fairing},
    http::{Header, hyper::header::{ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_ORIGIN}},
    response::websocket::{WebsocketMessage, WebsocketChannel, CreateWebsocket, Websocket},
};
use rocket::http::ext::IntoCollection;
use rocket::http::hyper::body::Bytes;
use crate::model::{QueryRoot, SubscriptionRoot};
use tokio_stream::{Stream};
use futures::{Future, pin_mut, StreamExt, TryFutureExt};

pub type GraphQLSchemaValue = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;
pub type GraphQLSchema = Arc<GraphQLSchemaValue>;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/playground")]
fn graphiql() -> content::RawHtml<String> {
    content::RawHtml(GraphiQLSource::build().endpoint("/graphql").subscription_endpoint("/graphql/subscription").finish())
}

#[get("/graphql?<query..>")]
async fn graphql_query(schema: &State<GraphQLSchema>, query: GraphQLQuery) -> GraphQLResponse {
    let meep = schema.inner().deref();
    query.execute(schema.inner().deref()).await
}

#[post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_request(
    schema: &State<GraphQLSchema>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request.execute(&*schema.inner().deref()).await
}

struct MyConfig {
    count: AtomicUsize,
}

/*#[get("/graphql/subscription")]
async fn graphql_subscription(state: &State<MyConfig>, mut shutdown: Shutdown) -> EventStream![] {
    let counter = AtomicUsize::new(0);
    let stream = EventStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            let count = counter.fetch_add(1, Ordering::Relaxed);
            select! {
                _ = interval.tick() => {
                    println!("ping! {}", count);
                    yield Event::data(format!("ping! {}", count));
                },
                _ = &mut shutdown => {
                    println!("shutdown!");
                    break;
                }
            };
        }
    };
    stream.heartbeat(Duration::from_secs(20))
}*/

use async_stream::stream;
use rocket::form::validate::len;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;

/*struct TestSocket {
    schema: GraphQLSchema,
    protocol: async_graphql::http::WebSocketProtocols,
}

#[rocket::async_trait]
impl rocket::Upgrade<'static> for TestSocket {
    async fn start(&self, mut upgraded: rocket::http::hyper::upgrade::Upgraded) {
        // let ws = self.websocket.unwrap();

        upgraded.write_all(b"meep").await;
        println!("Sent!!!");

        let mut vec = Vec::new();
        upgraded.read_to_end(&mut vec).await;
        println!("recieved: {:?}", String::from_utf8(vec));

        /*let mut buf = vec![0; 128];
        loop {
            let data = upgraded.(&mut buf).await;
            //let meep = buf.clone();
            if let Ok(len) = data {
                if len > 0 {
                    println!("text: {:?}", &buf[..len]);
                }
            }
        }*/

        let stream = stream!{
            println!("meep");
            yield &[0; 10]
        };

        /*let mut ws = async_graphql::http::WebSocket::new(
            self.schema.deref().clone(),
            stream,
            self.protocol
        ).on_connection_init(|_: serde_json::Value| -> Ready<async_graphql::Result<_>> {
            futures_util::future::ready(Ok(Data::default()))
        }).map(|msg| match msg {
            async_graphql::http::WsMessage::Text(text) => {
                println!("text: {}", text.as_str());
            },
            async_graphql::http::WsMessage::Close(code, status) => {
                println!("code: {:?} status: {:?}", code, status.as_str());
            },
        }).map(Ok).forward(futures::sink()).await;*/
    }
}

impl<'r> rocket::response::Responder<'r, 'r> for TestSocket {
    fn respond_to(mut self, req: &rocket::Request<'_>) -> rocket::response::Result<'r> {
        self.protocol = async_graphql::http::WebSocketProtocols::from_str(req.headers().get_one("sec-websocket-protocol").unwrap_or("")).unwrap_or(WebSocketProtocols::SubscriptionsTransportWS);
        println!("protocol: {:?}", self.protocol);
        rocket::Response::build()
            .status(rocket::http::Status::SwitchingProtocols)
            .raw_header("Connection", "upgrade")
            .raw_header("Upgrade", "websocket")
            .upgrade(Some(Box::new(self)))
            .ok()
    }
}

#[get("/graphql/subscription")]
async fn graphql_subscription(schema: &State<GraphQLSchema>, state: &State<MyConfig>) -> TestSocket {
    TestSocket{
        schema: schema.inner().clone(),
        protocol: WebSocketProtocols::SubscriptionsTransportWS,
    }
}*/


#[get("/graphql/subscription")]
async fn graphql_subscription(schema: &State<GraphQLSchema>, state: &State<MyConfig>) -> Websocket<impl Fn(WebsocketChannel) -> Box<dyn futures::Future<Output = ()> + Send + Unpin>> {
    let websocket = CreateWebsocket! {
        let mut ch: WebsocketChannel = ch;

        //let input_stream = stream!{
            tokio::select!{
                _ = async {
                    while let Some(msg) = ch.next().await {
                        println!("Message: {msg:?}")
                    }
                } => {},
            }
        //};
    };

    //let graphql = async_graphql::http::WebSocket::new(schema, input_stream, Protocols::);
    websocket
}

/*#[get("/graphql/subscription")]
async fn graphql_subscription(schema: &State<GraphQLSchema>, state: &State<MyConfig>) -> Websocket<impl Fn(WebsocketChannel) -> Box<dyn futures::Future<Output = ()> + Send + Unpin>> {
    let websocket = accept_async(request).await.unwrap();

    let (incoming, outgoing) = websocket.split();
    let mut incoming = incoming.fuse();
    //let mut outgoing = outgoing.fuse();

    tokio::select! {
        result = async {
            loop {
                let message = incoming.next().await.unwrap().unwrap();
                let message = match message {
                    Message::Text(text) => text,
                    _ => continue,
                };
                let message = format!("{}: {}", message);
                outgoing.send();
                tx.send(Message::Text(message.clone())).unwrap();
            }
        } => {
        }
        result = async {
            while let Some(message) = rx.recv().await {
                outgoing.send(message).await.unwrap();
            }
        } => {}
    }
}*/

#[launch]
fn rocket() -> _ {
    let schema = Arc::new(Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot).finish());

    rocket::build().manage(schema).manage(MyConfig{count: AtomicUsize::new(0)}).attach(AdHoc::on_response("CORS", |_, res| {
        Box::pin(async move {
            res.set_header(Header::new(ACCESS_CONTROL_ALLOW_ORIGIN.as_str(), "*"));
            res.remove_header(ACCESS_CONTROL_ALLOW_CREDENTIALS.as_str());
        })
    })).mount("/", routes![index, graphql_query, graphql_request, graphql_subscription, graphiql])
}

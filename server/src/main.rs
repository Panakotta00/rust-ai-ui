#![feature(async_closure)]

#[macro_use]
extern crate rocket;

pub mod model;

use crate::model::{QueryRoot, SubscriptionRoot};
use async_graphql::futures_util::future::Ready;
use async_graphql::{futures_util, http::GraphiQLSource, Data, EmptyMutation, Schema};
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use async_stream::stream;
use futures::{Future, StreamExt};
use rocket::request::{FromRequest, Outcome};
use rocket::{
	fairing::AdHoc,
	get,
	http::Header,
	response::content,
	response::websocket::{Websocket, WebsocketChannel, WebsocketMessage},
	routes, State,
};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use rocket::fs::FileServer;

pub type GraphQLSchemaValue = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;
pub type GraphQLSchema = GraphQLSchemaValue;

#[get("/playground")]
fn graphiql() -> content::RawHtml<String> {
	content::RawHtml(
		GraphiQLSource::build()
			.endpoint("/graphql")
			.subscription_endpoint("/graphql")
			.finish(),
	)
}

#[get("/graphql?<query..>")]
async fn graphql_query(schema: &State<GraphQLSchema>, query: GraphQLQuery) -> GraphQLResponse {
	let _meep = schema.inner().deref();
	query.execute(schema.inner().deref()).await
}

#[post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_request(
	schema: &State<GraphQLSchema>,
	request: GraphQLRequest,
) -> GraphQLResponse {
	request.execute(&*schema.inner().deref()).await
}

#[options("/graphql")]
async fn graphql_request_opt() -> &'static str {
	"Hello World"
}

struct MyConfig {
	count: AtomicUsize,
}

struct WebSocketProtocol(async_graphql::http::WebSocketProtocols);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebSocketProtocol {
	type Error = ();

	async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
		let protocol = request.headers().get_one("sec-websocket-protocol").map_or(
			Some(async_graphql::http::WebSocketProtocols::GraphQLWS),
			|str| async_graphql::http::WebSocketProtocols::from_str(str).ok(),
		);
		match protocol {
			None => Outcome::Failure((rocket::http::Status::BadRequest, ())),
			Some(protocol) => Outcome::Success(WebSocketProtocol(protocol)),
		}
	}
}

enum WebsocketHandling {
	InputMsg(Option<WebsocketMessage>),
	OutputMsg(Option<WebsocketMessage>),
}

struct MyResponder<F> {
	inner: Websocket<F>,
}

impl<'a, F> rocket::response::Responder<'a, 'a> for MyResponder<F>
where
	F: Fn(WebsocketChannel) -> Box<dyn Future<Output = ()> + Send + Unpin>
		+ Sync
		+ Send
		+ 'a + 'static,
{
	fn respond_to(self, req: &'a rocket::Request<'_>) -> rocket::response::Result<'a> {
		let response = self.inner.respond_to(req);
		response.map(|mut m| {
			if let Some(protocol) = req.headers().get_one("sec-websocket-protocol") {
				m.set_raw_header("sec-websocket-protocol", protocol);
			}
			m
		})
	}
}

#[get("/graphql")]
async fn graphql_subscription(
	schema: &State<GraphQLSchema>,
	_state: &State<MyConfig>,
	ws_protocol: WebSocketProtocol,
) -> MyResponder<impl Fn(WebsocketChannel) -> Box<dyn futures::Future<Output = ()> + Send + Unpin>>
{
	let schema = schema.deref().clone();
	let protocol = ws_protocol.0;
	let websocket = Websocket::create(move |ch: WebsocketChannel| {
		let schema = schema.clone();
		std::boxed::Box::new(std::boxed::Box::pin(async move {
			println!("begin");
			let mut ch = ch;

			let (output_s, mut output_r) = async_channel::unbounded();

			let input_stream = stream! {
				loop {
					let input_msg = ch.next();
					let output_msg = output_r.next();
					match tokio::select!{
						v = input_msg => WebsocketHandling::InputMsg(v),
						v = output_msg => WebsocketHandling::OutputMsg(v),
					} {
						WebsocketHandling::InputMsg(Some(msg)) => {
							let data = msg.into_data();
							yield data;
						},
						WebsocketHandling::OutputMsg(Some(msg)) => {
							ch.send(msg).await;
						},
						_ => break,
					}
				}

				while match ch.next().await {
					Some(msg) => {
						let data = msg.into_data();
						yield data;
						true
					},
					None => {
						false
					}
				} {}
			};

			let mut graphql_socket = Box::pin(
				async_graphql::http::WebSocket::new(schema, input_stream, protocol)
					.connection_data(Data::default())
					.on_connection_init(
						|_val: serde_json::Value| -> Ready<Result<Data, async_graphql::Error>> {
							futures_util::future::ready(Ok(Data::default()))
						},
					),
			);
			while match graphql_socket.next().await {
				Some(msg) => match msg {
					async_graphql::http::WsMessage::Text(text) => {
						let _ = output_s.send(WebsocketMessage::text(text)).await;
						true
					}
					async_graphql::http::WsMessage::Close(code, text) => {
						let _ = output_s
							.send(WebsocketMessage::close(Some((code, text))))
							.await;
						false
					}
				},
				None => false,
			} {}
			println!("end");
		}))
	});
	MyResponder { inner: websocket }
}

#[launch]
fn rocket() -> _ {
	let schema: GraphQLSchema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot).finish();

	rocket::build()
		.manage(schema)
		.manage(MyConfig {
			count: AtomicUsize::new(0),
		})
		.attach(AdHoc::on_response("CORS", |_, res| {
			Box::pin(async move {
				res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
				res.set_header(Header::new(
					"Access-Control-Allow-Methods",
					"POST, GET, PATCH, OPTIONS",
				));
				res.set_header(Header::new("Access-Control-Allow-Headers", "*"));
				res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
			})
		}))
		.mount(
			"/",
			FileServer::from("./website/dist"))
		.mount(
			"/",
			routes![
				graphql_query,
				graphql_request,
				graphql_subscription,
				graphiql,
				graphql_request_opt
			],
		)
}

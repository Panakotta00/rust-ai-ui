use crate::graphql::queries::MySubscription;
use futures::StreamExt;
use log::info;
use log::Level;
use wasm_bindgen::UnwrapThrowExt;

#[cynic::schema_for_derives(file = r#"schema.graphql"#, module = "schema")]
mod queries {
	use super::schema;

	#[derive(cynic::QueryFragment, Debug)]
	#[cynic(graphql_type = "SubscriptionRoot")]
	pub struct MySubscription {
		#[arguments(step = 7)]
		pub integers: i32,
	}
}

mod schema {
	cynic::use_schema!(r#"schema.graphql"#);
}

fn build_query() -> cynic::StreamingOperation<'static, MySubscription> {
	use cynic::SubscriptionBuilder;

	MySubscription::build(())
}

pub async fn do_graphql_stuff<F>(on_msg: F)
where
	F: Fn(i32),
{
	console_log::init_with_level(Level::Debug);

	let (ws, wsio) = ws_stream_wasm::WsMeta::connect(
		"/graphql",
		Some(vec!["graphql-transport-ws"]),
	)
	.await
	.map_err(|e| info!("{e}"))
	.expect_throw("assumed connection succeeds");

	info!("connected");

	let (sink, stream) = graphql_ws_client::wasm_websocket_combined_split(ws, wsio).await;

	let mut client = graphql_ws_client::CynicClientBuilder::new()
		.build(stream, sink, async_executors::AsyncStd)
		.await
		.unwrap();

	let mut stream = client.streaming_operation(build_query()).await.unwrap();
	info!("Running subscription");
	while let Some(item) = stream.next().await {
		if let Ok(v) = item {
			on_msg(v.data.unwrap().integers);
		}
	}
}

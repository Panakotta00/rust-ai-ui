use async_graphql::{Object, Subscription};
use tokio_stream::*;

pub struct QueryRoot;
pub struct SubscriptionRoot;

#[Object]
impl QueryRoot {
	async fn add(&self, a: i32, b: i32) -> i32 {
		a + b
	}
}

#[Subscription]
impl SubscriptionRoot {
	async fn integers(
		&self,
		#[graphql(default = 1)] step: i32,
	) -> impl futures_core::Stream<Item = i32> {
		let mut value = 0;
		tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
			core::time::Duration::from_secs(1),
		))
		.map(move |_| {
			value += step;
			value
		})
	}
}

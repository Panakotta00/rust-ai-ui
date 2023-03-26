mod graphql;

use crate::graphql::do_graphql_stuff;
use futures::StreamExt;

use web_sys::console::log_1;
use web_sys::HtmlInputElement;

use yew::prelude::*;

#[derive(Properties, PartialEq)]
struct MessageListProps {
	messages: Vec<String>,
}

#[function_component(MessageList)]
fn message_list(props: &MessageListProps) -> Html {
	props
		.messages
		.iter()
		.map(|message| {
			let mark = Html::from_html_unchecked(markdown::to_html(message).into());
			html! {
				<>
					{mark}
				</>
			}
		})
		.collect()
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
		let link = ctx.link().clone();
		wasm_bindgen_futures::spawn_local(async move {
			do_graphql_stuff(move |num| {
				link.send_message(AppMsg::NewMsg(format!("Some new Number: {num}")));
			})
			.await;
		});

		Self {
			messages: Default::default(),
			input: NodeRef::default(),
			count: 0,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			AppMsg::SSE(msg) => {
				self.messages.push(msg);
				true
			}
			AppMsg::NewMsg(msg) => {
				self.messages.push(msg);
				true
			}
			AppMsg::Increment => {
				self.count += 1;
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let _input = NodeRef::default();

		let submit_new_message = {
			let input = self.input.clone();
			ctx.link().callback(move |event: SubmitEvent| {
				event.prevent_default();
				let msg = input.cast::<HtmlInputElement>().unwrap().value();
				AppMsg::NewMsg(msg.clone())
			})
		};

		let increment_click = { ctx.link().callback(move |_| AppMsg::Increment) };

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

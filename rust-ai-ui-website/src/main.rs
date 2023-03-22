use web_sys::HtmlInputElement;
use yew::prelude::*;

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

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let messages = use_state(|| vec!["Fuck you!", "No, fuck you!", "Shut the fuck up!"].iter().map(|v| v.to_string()).collect::<Vec<_>>());
    let input = NodeRef::default();

    let submit_new_message = {
        let messages = messages.clone();
        let input = input.clone();
        Callback::from(move |event: SubmitEvent| {
            println!("meep");
            event.prevent_default();
            let mut msgs = messages.to_vec();
            msgs.push(input.cast::<HtmlInputElement>().unwrap().value());
            messages.set(msgs);
        })
    };

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
            <MessageList messages={(*messages).clone()} />
            <form onsubmit={submit_new_message}>
                <input ref={&input} />
            </form>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

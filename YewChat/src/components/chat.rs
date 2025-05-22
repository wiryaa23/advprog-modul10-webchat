use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let current_user = ctx.link().context::<User>(Callback::noop()).unwrap().0.username.borrow().clone();

        html! {
            <div class="flex w-screen">
                <div class="flex-none w-56 h-screen bg-blue-100"> // <- updated to lighter blue
                    <div class="text-xl p-3 font-semibold text-blue-800">{"👥 Active Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white rounded-lg p-2 hover:bg-blue-200 transition-all cursor-pointer">
                                    <img class="w-10 h-10 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    <div class="flex-grow pl-3 pt-1">
                                        <div class="text-sm font-medium text-gray-700">{u.name.clone()}</div>
                                        <div class="text-xs text-gray-400">{"Hi there!"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>

                <div class="grow h-screen flex flex-col bg-white">
                    <div class="w-full h-14 border-b-2 border-blue-200">
                        <div class="text-xl p-3 font-semibold text-blue-700">{"💬 Chat Room"}</div>
                    </div>

                    <div class="w-full grow overflow-auto px-6 py-4 space-y-4">
                        {
                            self.messages.iter().map(|m| {
                                let is_self = m.from == current_user;

                                let bubble_class = if is_self {
                                    "ml-auto bg-blue-200 text-right rounded-tl-lg rounded-bl-lg rounded-br-lg"
                                } else {
                                    "mr-auto bg-gray-100 text-left rounded-tr-lg rounded-bl-lg rounded-br-lg"
                                };

                                html! {
                                    <div class={format!("flex items-end max-w-[60%] p-2 {}", bubble_class)}>
                                        {
                                            if !is_self {
                                                if let Some(u) = self.users.iter().find(|u| u.name == m.from) {
                                                    html! {
                                                        <img class="w-8 h-8 rounded-full mr-2" src={u.avatar.clone()} alt="avatar"/>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                        <div class="text-sm">
                                            <div class="font-semibold text-blue-800">{m.from.clone()}</div>
                                            <div class="text-xs text-gray-700 mt-1">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! {
                                                            <img class="mt-2 max-w-full rounded-md" src={m.message.clone()} />
                                                        }
                                                    } else {
                                                        html! {
                                                            { m.message.clone() }
                                                        }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    <div class="w-full h-16 flex px-4 py-2 items-center border-t-2 border-blue-100 bg-gray-50">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Type a message..."
                            class="flex-grow py-2 px-4 bg-white border border-gray-300 rounded-full outline-none focus:ring-2 focus:ring-blue-300"
                            required=true
                        />
                        <button onclick={submit} class="ml-3 p-3 bg-blue-600 hover:bg-blue-700 text-white rounded-full">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 fill-current">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
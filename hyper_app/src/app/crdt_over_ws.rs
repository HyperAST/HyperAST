use std::sync::{Arc, RwLock};

#[cfg(target_arch = "wasm32")]
use async_executors::JoinHandle;
use autosurgeon::{reconcile, Hydrate, Reconcile};
use egui_addon::code_editor::generic_text_buffer::TextBuffer;
use futures_util::{Future, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use tokio::task::JoinHandle;

#[derive(Default, Debug, Reconcile, Hydrate)]
pub(crate) struct Quote {
    pub(crate) text: autosurgeon::Text,
}

impl<'de> Deserialize<'de> for Quote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use std::fmt;
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a string containing at least {} bytes", 0)
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(s.to_owned())
            }
        }
        deserializer.deserialize_string(V).map(|x| x.into())
    }
}

impl Serialize for Quote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self.text.as_str())
    }
}

impl<S: Into<String>> From<S> for Quote {
    fn from(value: S) -> Self {
        Quote {
            text: value.into().into(),
        }
    }
}

impl egui_addon::code_editor::generic_text_buffer::AsText for Quote {
    fn text(&self) -> &str {
        self.text.as_str()
    }
}

impl TextBuffer for Quote {
    type Ref = Quote;

    fn is_mutable(&self) -> bool {
        true
    }

    fn as_reference(&self) -> &Self::Ref {
        self
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        let l = text.len();
        self.text.splice(char_index, 0, text);
        l
    }

    fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
        self.text
            .splice(char_range.start, char_range.end - char_range.start, "");
    }
}

pub(super) struct WsCont(tokio_tungstenite_wasm::WebSocketStream);
unsafe impl Send for WsCont {}

pub(super) struct WsChannel<S> {
    ws: WsState,
    pub data: Arc<RwLock<S>>,
    pub timer: f32,
}

/// state realtive to server ie. no p2p
pub(super) type DocSharingState = (automerge::AutoCommit, automerge::sync::State);

pub(super) type WsDoc = WsChannel<DocSharingState>;

type User = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(super) struct SharedDocView {
    pub(super) owner: User,
    pub(super) name: String,
    // TODO show users currently on shared doc
    writers: Vec<User>,
    pub(super) id: usize,
}

pub(super) type WsDocsDb = WsChannel<(Option<usize>, Vec<Option<SharedDocView>>)>;

#[derive(Default)]
enum WsState {
    Init(poll_promise::Promise<tokio_tungstenite_wasm::Result<WsCont>>),
    Error(tokio_tungstenite_wasm::Error),
    Setup(
        futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
        H,
    ),
    #[default]
    Empty,
}

impl<S> WsChannel<S> {
    pub(super) fn with_data(rt: &Rt, who: User, ctx: egui::Context, url: String, data: S) -> Self {
        let (s, p) = poll_promise::Promise::new();
        rt.spawn(async move {
            s.send(WsChannel::<S>::make_ws_async(who, url).await);
            ctx.request_repaint();
        });
        WsChannel {
            ws: WsState::Init(p),
            data: Arc::new(RwLock::new(data)),
            timer: 0.0,
        }
    }
    async fn make_ws_async(who: User, url: String) -> tokio_tungstenite_wasm::Result<WsCont> {
        wasm_rs_dbg::dbg!(&url);
        match tokio_tungstenite_wasm::connect(url).await {
            Ok(stream) => {
                wasm_rs_dbg::dbg!("Handshake for client {} has been completed", who);
                Ok(WsCont(stream))
            }
            Err(e) => {
                wasm_rs_dbg::dbg!(
                    "WebSocket handshake for client {who} failed with {e}!",
                    &who,
                    &e
                );
                Err(e)
            }
        }
    }
    pub(super) fn setup_atempt<F>(&mut self, receiver_f: F, rt: &Rt) -> Result<(), String>
    where
        F: FnOnce(
            futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
            futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
        ) -> H,
    {
        // NOTE could replace Empty variant by the following default value
        // tokio_tungstenite_wasm::Error::from(http::status::StatusCode::from_u16(42).unwrap_err());
        match std::mem::take(&mut self.ws) {
            WsState::Init(prom) => {
                match prom.try_take() {
                    Ok(Ok(ws)) => {
                        let (mut sender, receiver) = ws.0.split();
                        let (s, mut r) = futures::channel::mpsc::channel(50);
                        rt.spawn(async move {
                            wasm_rs_dbg::dbg!();
                            while let Some(x) = r.next().await {
                                wasm_rs_dbg::dbg!();
                                if let Err(err) = sender.send(x).await {
                                    wasm_rs_dbg::dbg!(err);
                                }
                            }
                        });
                        self.ws = WsState::Setup(s.clone(), receiver_f(s, receiver))
                    }
                    Ok(Err(err)) => {
                        let error = err.to_string();
                        self.ws = WsState::Error(err);
                        return Err(error);
                    }
                    Err(prom) => self.ws = WsState::Init(prom),
                };
                Ok(())
            }
            WsState::Error(err) => {
                let error = err.to_string();
                self.ws = WsState::Error(err);
                Err(error)
            }
            WsState::Setup(s, r) => {
                self.ws = WsState::Setup(s, r);
                Ok(())
            }
            WsState::Empty => panic!("unrecoverable state"),
        }
    }

    pub(crate) fn is_connected(&self) -> bool {
        match &self.ws {
            WsState::Init(_) => false,
            WsState::Error(_) => false,
            WsState::Setup(_, _) => true,
            WsState::Empty => unreachable!(),
        }
    }
}

impl WsDoc {
    pub(super) fn new(rt: &Rt, who: User, ctx: egui::Context, url: String) -> Self {
        let data = (automerge::AutoCommit::new(), automerge::sync::State::new());
        WsChannel::with_data(rt, who, ctx, url, data)
    }

    pub(crate) fn changed(&mut self, rt: &Rt, quote: &mut impl Reconcile) {
        wasm_rs_dbg::dbg!();
        let (doc, sync_state): &mut (_, _) = &mut self.data.write().unwrap();
        if let Err(e) = reconcile(doc, &*quote) {
            log::warn!(
                "failed to reconcile while updating local state of CRDT {}",
                e
            );
        };
        match &mut self.ws {
            WsState::Init(_) => (),
            WsState::Error(_) => (),
            WsState::Setup(sender, _) => {
                use automerge::sync::SyncDoc;
                if let Some(x) = doc.sync().generate_sync_message(sync_state) {
                    let x = tokio_tungstenite_wasm::Message::Binary(x.encode().to_vec());
                    let mut sender = sender.clone();
                    rt.spawn(async move {
                        sender.send(x).await.unwrap();
                    });
                } else {
                    wasm_rs_dbg::dbg!();
                }
            }
            WsState::Empty => panic!(),
        };
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum DbMsgToServer {
    Create { name: String },
    User { name: String },
}
impl WsDocsDb {
    pub(super) fn new(rt: &Rt, who: User, ctx: egui::Context, url: String) -> Self {
        let channel = WsChannel::with_data(rt, who, ctx, url, (None, vec![]));
        // match &mut channel.ws {
        //     WsState::Init(_) => (),
        //     WsState::Error(_) => (),
        //     WsState::Setup(sender, _) => {
        //         let name = "42".to_string();
        //         let msg = DbMsgToServer::User { name };
        //         let msg = serde_json::to_string(&msg).unwrap();
        //         let x = tokio_tungstenite_wasm::Message::Text(msg);
        //         let mut sender = sender.clone();
        //         rt.spawn(async move {
        //             sender.send(x).await.unwrap();
        //         });
        //     }
        //     WsState::Empty => panic!(),
        // };

        channel
    }

    pub(crate) fn create_doc_atempt(&mut self, rt: &Rt, name: String, quote: &mut impl Reconcile) {
        wasm_rs_dbg::dbg!();
        // let docs: &mut Vec<_> = &mut self.data.write().unwrap();
        // if let Err(e) = reconcile(doc, &*quote) {
        //     log::warn!(
        //         "failed to reconcile while updating local state of CRDT {}",
        //         e
        //     );
        // };
        match &mut self.ws {
            WsState::Init(_) => (),
            WsState::Error(_) => (),
            WsState::Setup(sender, _) => {
                let msg = DbMsgToServer::Create { name };
                let msg = serde_json::to_string(&msg).unwrap();
                let x = tokio_tungstenite_wasm::Message::Text(msg);
                let mut sender = sender.clone();
                rt.spawn(async move {
                    sender.send(x).await.unwrap();
                });
            }
            WsState::Empty => panic!(),
        };
    }
}

// # Cross platform async utils

#[derive(Clone)]
pub(super) struct Rt(#[cfg(not(target_arch = "wasm32"))] pub(super) Arc<tokio::runtime::Runtime>);

impl Default for Rt {
    fn default() -> Self {
        Self(
            #[cfg(not(target_arch = "wasm32"))]
            Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            ),
        )
    }
}

pub(super) struct H(#[cfg(not(target_arch = "wasm32"))] JoinHandle<()>);

impl Rt {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn spawn<F>(&self, future: F) -> H
    where
        F: Future<Output = ()> + Send + 'static,
    {
        H(self.0.spawn(future))
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn spawn<F>(&self, future: F) -> H
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future);
        H()
    }
}

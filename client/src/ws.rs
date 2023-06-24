use automerge::sync::SyncDoc;
use axum::TypedHeader;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use std::net::SocketAddr;
use std::ops::DerefMut;

// allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

// allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};

use crate::SharedState;

#[debug_handler]
pub(crate) async fn connect_db(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> impl IntoResponse {
    dbg!(&addr);
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected to db.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket_db(socket, addr, state.clone()))
}
type User = String;

pub(crate) struct SharedDoc {
    owner: User,
    name: String,
    writers: Vec<User>,
    doc: automerge::AutoCommit,
    members: Vec<tokio::sync::mpsc::Sender<Option<Vec<u8>>>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SharedDocView {
    owner: User,
    name: String,
    writers: Vec<User>,
    id: usize,
}
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) struct SharedDocs {
    pub docs: Arc<RwLock<Vec<Option<Arc<RwLock<SharedDoc>>>>>>,
    s: tokio::sync::broadcast::Sender<DbMsgOut>,
    r: tokio::sync::broadcast::Receiver<DbMsgOut>,
}
impl Default for SharedDocs {
    fn default() -> Self {
        let (s, r) = tokio::sync::broadcast::channel(50);
        let docs = Default::default();
        Self { docs, s, r }
    }
}
struct DocHandle(usize);

#[derive(Deserialize, Serialize, Debug, Clone)]
enum DbMsgIn {
    Create { name: String },
    User { name: String },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum DbMsgOut {
    Add(SharedDocView),
    AddWriter(usize, User),
    RmWriter(usize, User),
    // Rename(usize, String),
    Reset { all: Vec<SharedDocView> },
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket_db(socket: WebSocket, who: SocketAddr, state: SharedState) {
    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();
    let s = state.doc2.s.clone();
    let mut r = state.doc2.r.resubscribe();

    let user;
    dbg!(who);
    match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_str(&text) {
                Ok(DbMsgIn::User { name }) => {
                    user = name;
                }
                Ok(x) => panic!("{:?}", x),
                Err(e) => {
                    panic!("{:?}", e)
                }
            };
        }
        Some(Ok(Message::Binary(d))) => {
            panic!()
        }
        Some(Ok(Message::Close(c))) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {} somehow sent close message without CloseFrame", who);
            }
            // break;
            return;
        }
        Some(Err(e)) => panic!("{:?}", e),
        Some(e) => panic!("{:?}", e),
        None => todo!(),
    };
    dbg!(who);
    let all = state
        .doc2
        .docs
        .read()
        .unwrap()
        .iter()
        .enumerate()
        .filter_map(|(i, x)| {
            x.as_ref().map(|y| {
                let y = y.read().unwrap();
                SharedDocView {
                    owner: y.owner.clone(),
                    name: y.name.clone(),
                    writers: y.writers.clone(),
                    id: i,
                }
            })
        })
        .collect();
    sender
        .send(Message::Text(
            serde_json::to_string(&DbMsgOut::Reset { all }).unwrap(),
        ))
        .await
        .unwrap();
    dbg!(who);
    let usr = user.clone();
    let mut send_task = tokio::spawn(async move {
        let mut cnt = 0;
        loop {
            cnt += 1;
            match r.recv().await {
                Ok(x) //if &x.owner != &usr 
                => {
                    sender
                        .send(Message::Text(serde_json::to_string(&x).unwrap()))
                        .await
                        .unwrap();
                }
                // Ok(x) => {}
                Err(e) => {
                    dbg!(e);
                    break;
                }
            };
        }
        cnt
    });
    let sen = s.clone();
    // This second task will receive messages from current client
    let mut recv_task = tokio::spawn(async move {
        let user = user.clone();
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            dbg!(&msg);
            cnt += 1;
            match msg {
                Message::Text(text) => {
                    let msg: DbMsgIn = match serde_json::from_str(&text) {
                        Ok(x) => x,
                        Err(e) => {
                            dbg!(e);
                            continue;
                        }
                    };
                    let msg = match msg {
                        DbMsgIn::Create { name } => {
                            dbg!(who);
                            let new = SharedDoc {
                                owner: user.clone(),
                                name: name.clone(),
                                writers: vec![],
                                doc: automerge::AutoCommit::new(),
                                members: vec![],
                            };
                            let docs = &mut state.doc2.docs.write().unwrap();
                            let id = docs.len();
                            docs.push(Some(Arc::new(RwLock::new(new))));
                            DbMsgOut::Add(SharedDocView {
                                owner: user.clone(),
                                name,
                                writers: vec![],
                                id,
                            })
                        }
                        DbMsgIn::User { .. } => panic!(),
                    };

                    match sen.send(msg) {
                        Ok(n) => log::info!("broadcasted to {} clients", n),
                        Err(e) => log::error!("failed to boadcast due to {}", e),
                    }
                }
                Message::Binary(d) => {
                    dbg!()
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        println!(
                            ">>> {} sent close with code {} and reason `{}`",
                            who, cf.code, cf.reason
                        );
                    } else {
                        println!(">>> {} somehow sent close message without CloseFrame", who);
                    }
                    break;
                }

                Message::Pong(v) => {
                    println!(">>> {} sent pong with {:?}", who, v);
                }
                // You should never need to manually handle Message::Ping, as axum's websocket library
                // will do so for you automagically by replying with Pong and copying the v according to
                // spec. But if you need the contents of the pings you can see them here.
                Message::Ping(v) => {
                    println!(">>> {} sent ping with {:?}", who, v);
                }
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{} messages sent to {}", a, who),
                Err(a) => println!("Error sending messages {:?}", a)
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {} messages", b),
                Err(b) => println!("Error receiving messages {:?}", b)
            }
            // send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {} destroyed", who);
}

#[debug_handler]
pub(crate) async fn connect_doc(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Path(session_id): axum::extract::Path<usize>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> impl IntoResponse {
    dbg!(&addr);
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state.clone(), session_id))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr, state: SharedState, session: usize) {
    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();
    let (s, mut r) = tokio::sync::mpsc::channel(50);
    let user;
    let position = match receiver.next().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str(&text) {
            Ok(DbMsgIn::User { name }) => {
                user = name;
                if let Some(x) = state.doc2.docs.write().unwrap().get(session) {
                    if let Some(x) = x {
                        let mut shared_doc = x.write().unwrap();
                        let position = shared_doc.members.len();
                        shared_doc.members.push(s.clone());
                        shared_doc.writers.push(user);
                        position
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            Ok(x) => panic!("{:?}", x),
            Err(e) => {
                panic!("{:?}", e)
            }
        },
        Some(Ok(Message::Binary(d))) => {
            panic!()
        }
        Some(Ok(Message::Close(c))) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {} somehow sent close message without CloseFrame", who);
            }
            // break;
            return;
        }
        Some(Err(e)) => panic!("{:?}", e),
        Some(e) => panic!("{:?}", e),
        None => todo!(),
    };
    let mut sync_state = automerge::sync::State::new();

    let _state = state.clone();
    // Spawn a task that handle both syncing the shared automerge doc and send updates to clients
    let mut send_task = tokio::spawn(async move {
        let state = _state;
        {
            let msg = {
                let vecs = state.doc2.docs.read().unwrap();
                let shared = vecs[session].as_ref();
                let rw_lock_write_guard = &mut shared.unwrap().write().unwrap();
                let doc = &mut rw_lock_write_guard.deref_mut().doc;
                let msg = doc.sync().generate_sync_message(&mut sync_state);
                msg
            };
            if let Some(a) = msg {
                dbg!(who);
                if let Err(err) = sender.send(Message::Binary(a.encode())).await {
                    // TODO match specifically ConnectionClosed ?
                    // err.into_inner().is::<axum::BoxError>();
                    dbg!(err);
                }
            }
        }
        let mut cnt = 0;
        loop {
            cnt += 1;
            let mut recv = r.recv().await;
            let mut changed = false;
            if let Some(aaa) = &mut recv {
                if let Some(d) = aaa.take() {
                    dbg!(who);
                    if d.is_empty() {
                    } else {
                        let vecs = state.doc2.docs.read().unwrap();
                        let shared = vecs[session].as_ref();
                        let shared = &mut shared.unwrap().write().unwrap();
                        let shared = shared.deref_mut();
                        let message = automerge::sync::Message::decode(&d).unwrap();
                        shared.doc.sync()
                            .receive_sync_message(&mut sync_state, message)
                            .unwrap();
                    }
                    changed = true;
                }
            };
            match recv {
                // Ok(heads) => {
                Some(_) => {
                    dbg!(who);
                    let msg = {
                        let vecs = state.doc2.docs.read().unwrap();
                        let shared = vecs[session].as_ref();
                        let shared = &mut shared.unwrap().write().unwrap();
                        let shared = shared.deref_mut();
                        let msg = shared.doc.sync().generate_sync_message(&mut sync_state);
                        msg
                    };
                    if let Some(a) = msg {
                        dbg!(who);
                        if let Err(err) = sender.send(Message::Binary(a.encode())).await {
                            // TODO match specifically ConnectionClosed ?
                            // err.into_inner().is::<axum::BoxError>();
                            dbg!(err);
                            break;
                        }
                    } else if changed {
                        dbg!(who);
                        for i in 0.. {
                            if i == position {
                                continue;
                            }
                            let s = if let Some(x) = state.doc2.docs.write().unwrap().get(session) {
                                if let Some(x) = x {
                                    let mut shared_doc = x.write().unwrap();
                                    let Some(s) = &mut shared_doc.members.get_mut(i) else {break};
                                    s.clone()
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            };
                            dbg!(who);
                            if let Err(e) = s.send(None).await {
                                dbg!(e);
                            }
                        }
                    }
                }
                None => {
                    break;
                }
            };
        }
        cnt
    });

    // This second task will receive messages from current client
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            let who = who;
            match msg {
                Message::Text(t) => {
                    println!(">>> {} sent str: {:?}", who, t);
                }
                Message::Binary(d) => {
                    s.send(Some(d)).await.unwrap();
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        println!(
                            ">>> {} sent close with code {} and reason `{}`",
                            who, cf.code, cf.reason
                        );
                    } else {
                        println!(">>> {} somehow sent close message without CloseFrame", who);
                    }
                    break;
                }

                Message::Pong(v) => {
                    println!(">>> {} sent pong with {:?}", who, v);
                }
                // You should never need to manually handle Message::Ping, as axum's websocket library
                // will do so for you automagically by replying with Pong and copying the v according to
                // spec. But if you need the contents of the pings you can see them here.
                Message::Ping(v) => {
                    println!(">>> {} sent ping with {:?}", who, v);
                }
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{} messages sent to {}", a, who),
                Err(a) => println!("Error sending messages {:?}", a)
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {} messages", b),
                Err(b) => println!("Error receiving messages {:?}", b)
            }
            // send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {} destroyed", who);
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
#[debug_handler]
pub(crate) async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> impl IntoResponse {
    dbg!(&addr);
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket_automerge_sync(socket, addr, state.clone()))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket_automerge_sync(socket: WebSocket, who: SocketAddr, state: SharedState) {
    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();
    let (s, mut r) = tokio::sync::mpsc::channel(50);
    let position = state.doc.2.write().unwrap().len();
    state.doc.2.write().unwrap().push(s.clone());
    let mut sync_state = automerge::sync::State::new();

    let _state = state.clone();
    // Spawn a task that handle both syncing the shared automerge doc and send updates to clients
    let mut send_task = tokio::spawn(async move {
        let state = _state;
        {
            let msg = {
                let doc = &mut state.doc.0.write().unwrap();
                let msg = doc.sync().generate_sync_message(&mut sync_state);
                msg
            };
            if let Some(a) = msg {
                dbg!(who);
                if let Err(err) = sender.send(Message::Binary(a.encode())).await {
                    // TODO match specifically ConnectionClosed ?
                    // err.into_inner().is::<axum::BoxError>();
                    dbg!(err);
                }
            }
        }
        let mut cnt = 0;
        loop {
            cnt += 1;
            let mut recv = r.recv().await;
            let mut changed = false;
            if let Some(aaa) = &mut recv {
                if let Some(d) = aaa.take() {
                    dbg!(who);
                    if d.is_empty() {
                    } else {
                        let doc = &mut state.doc.0.write().unwrap();
                        let message = automerge::sync::Message::decode(&d).unwrap();
                        doc.sync()
                            .receive_sync_message(&mut sync_state, message)
                            .unwrap();
                    }
                    changed = true;
                }
            };
            match recv {
                // Ok(heads) => {
                Some(_) => {
                    dbg!(who);
                    let msg = {
                        let doc = &mut state.doc.0.write().unwrap();
                        let msg = doc.sync().generate_sync_message(&mut sync_state);
                        msg
                    };
                    if let Some(a) = msg {
                        dbg!(who);
                        if let Err(err) = sender.send(Message::Binary(a.encode())).await {
                            // TODO match specifically ConnectionClosed ?
                            // err.into_inner().is::<axum::BoxError>();
                            dbg!(err);
                            break;
                        }
                    } else if changed {
                        dbg!(who);
                        let len = state.doc.2.write().unwrap().len();
                        for i in 0..len {
                            if i == position {
                                continue;
                            }
                            let s = {
                                let s = &mut state.doc.2.write().unwrap()[i];
                                s.clone()
                            };
                            dbg!(who);
                            if let Err(e) = s.send(None).await {
                                dbg!(e);
                            }
                        }
                    }
                }
                None => {
                    break;
                }
            };
        }
        cnt
    });

    // This second task will receive messages from current client
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            let who = who;
            match msg {
                Message::Text(t) => {
                    println!(">>> {} sent str: {:?}", who, t);
                }
                Message::Binary(d) => {
                    s.send(Some(d)).await.unwrap();
                }
                Message::Close(c) => {
                    if let Some(cf) = c {
                        println!(
                            ">>> {} sent close with code {} and reason `{}`",
                            who, cf.code, cf.reason
                        );
                    } else {
                        println!(">>> {} somehow sent close message without CloseFrame", who);
                    }
                    break;
                }

                Message::Pong(v) => {
                    println!(">>> {} sent pong with {:?}", who, v);
                }
                // You should never need to manually handle Message::Ping, as axum's websocket library
                // will do so for you automagically by replying with Pong and copying the v according to
                // spec. But if you need the contents of the pings you can see them here.
                Message::Ping(v) => {
                    println!(">>> {} sent ping with {:?}", who, v);
                }
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{} messages sent to {}", a, who),
                Err(a) => println!("Error sending messages {:?}", a)
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {} messages", b),
                Err(b) => println!("Error receiving messages {:?}", b)
            }
            // send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {} destroyed", who);
}

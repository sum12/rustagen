use core::panic;
use std::io::{BufRead, StdoutLock, Write};

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,

    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

pub trait Node<S, P> {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn step(&mut self, input: Message<P>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, N, P>(init_state: S) -> anyhow::Result<()>
where
    P: DeserializeOwned,
    N: Node<S, P>,
{
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();
    let init_msg: Message<InitPayload> =
        serde_json::from_str(&stdin.next().expect("no input message on stdin")?)
            .context("init message could not be deserialized")?;

    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init")
    };

    let mut node: N = Node::from_init(init_state, init).context("node initialization failed")?;

    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,

        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &reply).context("serialize response to echo")?;
    stdout.write_all(b"\n").context("write trailing newline")?;

    //     let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();
    for line in stdin {
        let line = line.context("MaleStrom input could not be read")?;
        let line: Message<P> =
            serde_json::from_str(&line).context("MaleStrom input could not be deserialized")?;
        node.step(line, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}

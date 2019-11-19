#[macro_use]
extern crate lazy_static;

use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct Response {
    result: Value,
    id: isize,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct Event {
    method: String,
    params: Value,
}

impl Event {
    fn try_get_callframes(&self) -> Option<Vec<CallFrame>> {
        serde_json::from_value::<Vec<CallFrame>>(self.params["callFrames"].clone()).ok()
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CallFrame {
    function_name: String,
    location: Location,
    scope_chain: Vec<Scope>,
    url: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Location {
    column_number: usize,
    line_number: usize,
    script_id: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Scope {
    name: Option<String>,
    object: RemoteObject,
    r#type: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RemoteObject {
    r#type: String,
    object_id: String,
}

#[derive(Clone, Debug, PartialEq)]
enum MsgType {
    Response(Response),
    PauseEvent(Event),
    OtherEvent(Event),
}

lazy_static! {
    static ref MSG_ID: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
}

type Client = websocket::client::sync::Client<std::net::TcpStream>;

fn send_msg(msg: &str, wsclient: &mut Client) {
    let mut id = MSG_ID.lock().unwrap();
    let msg = format!(r#"{{"id": {}, {}}}"#, *id, msg);
    wsclient
        .send_message(&websocket::Message::text(&msg))
        .unwrap();
    *id += 1;
    println!("Sent : {}", msg);
}

fn wait_msg(wsclient: &mut Client) -> Option<MsgType> {
    use websocket::OwnedMessage;
    loop {
        let resp = wsclient.recv_message();
        match resp {
            Err(websocket::WebSocketError::NoDataAvailable) => return None,
            _ => {}
        }
        let resp = resp.unwrap();
        if let OwnedMessage::Text(i) = resp {
            let v = serde_json::from_str::<Response>(&i);
            if let Ok(e) = v {
                //println!("Resp: {}", i);
                println!("Response #{}", e.id);
                return Some(MsgType::Response(e));
            } else if let Ok(e) = serde_json::from_str::<Event>(&i) {
                //println!("Event: {}", i);
                println!("Event {}", e.method);
                if e.method == "Debugger.paused" {
                    //*LAST_PAUSE.lock().unwrap() = Some(e);
                    return Some(MsgType::PauseEvent(e));
                }
                return Some(MsgType::OtherEvent(e));
            }
        } else if let OwnedMessage::Close(_) = resp {
            return None;
        } else {
            println!("Unexpected kind of message : {:?}", resp);
        }
    }
}

fn init_debugger(mut c: &mut Client) {
    let init_msgs = [
        r#""method":"Runtime.enable""#,
        r#""method":"Debugger.enable""#,
        r#""method":"Runtime.runIfWaitingForDebugger""#,
    ];

    for msg in init_msgs.iter() {
        send_msg(msg, &mut c);
        loop {
            match wait_msg(&mut c).unwrap() {
                MsgType::Response(_) => break,
                _ => {}
            }
        }
    }
}

fn set_breakpoint_all(lines: usize, filename: &str, mut c: &mut Client) {
    for i in 0..lines {
        send_msg(
            &format!(
                r#""method":"Debugger.setBreakpointByUrl", "params":{{"lineNumber": {}, "urlRegex": ".*{}"}}"#,
                i, filename
            ),
            &mut c,
        );
        loop {
            match wait_msg(&mut c).unwrap() {
                MsgType::Response(_) => break,
                _ => {}
            }
        }
    }
}

fn jump_to_file(filename: &str, mut c: &mut Client) -> Event {
    loop {
        let evn;
        send_msg(r#""method":"Debugger.stepOver""#, &mut c);
        loop {
            match wait_msg(&mut c).unwrap() {
                MsgType::PauseEvent(e) => {
                    evn = e;
                    break;
                }
                _ => {}
            }
        }
        if evn.try_get_callframes().unwrap()[0].url.contains(filename) {
            return evn;
        }
    }
}

fn main() {
    // TODO: better message handling
    // e.g. use queue for event handling

    let addr = "ws://127.0.0.1:9229/graal-inspector";

    let mut c = websocket::ClientBuilder::new(addr)
        .unwrap()
        .connect_insecure()
        .unwrap();

    init_debugger(&mut c);

    // FIXME: read file and get line count
    set_breakpoint_all(14, "test.py", &mut c);

    let mut evn = jump_to_file("test.py", &mut c);

    {
        let mut f = std::fs::File::create("trace.txt").unwrap();
        use std::io::Write;
        'out: loop {
            let cf = evn.try_get_callframes().unwrap();
            println!("cf size: {}", cf.len());
            let cf0 = &cf[0];
            for sc in &cf0.scope_chain {
                send_msg(
                    &format!(
                        r#""method":"Runtime.getProperties", params: {{"objectId": {}}}"#,
                        sc.object.object_id
                    ),
                    &mut c,
                );
                loop {
                    match wait_msg(&mut c) {
                        Some(MsgType::Response(e)) => {
                            writeln!(f, "{:?} {:?} {:?}", cf0.location, sc.name, e.result).unwrap();
                            break;
                        }
                        Some(_) => {}
                        None => {
                            break 'out;
                        }
                    }
                }
            }
            writeln!(f).unwrap();

            send_msg(r#""method":"Debugger.stepOver""#, &mut c);
            loop {
                match wait_msg(&mut c) {
                    Some(MsgType::PauseEvent(e)) => {
                        evn = e;
                        break;
                    }
                    Some(_) => {}
                    None => {
                        break 'out;
                    }
                }
            }
        }
    }
}

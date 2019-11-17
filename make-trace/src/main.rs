use serde::Deserialize;
use websocket::client::sync::Client;
use websocket::ClientBuilder;
use websocket::Message;
use websocket::OwnedMessage;

use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

#[derive(Clone, Deserialize, Debug)]
struct Response {
    result: serde_json::Value,
    id: isize,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct Event {
    method: String,
    params: serde_json::Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MsgType {
    Response,
    PauseEvent,
    OtherEvent,
}

lazy_static! {
    static ref LAST_PAUSE: Mutex<Option<Event>> = Mutex::new(None);
    static ref MSG_ID: Mutex<usize> = Mutex::new(0);
}

fn send_msg(msg: &str, wsclient: &mut Client<std::net::TcpStream>) {
    let mut id = MSG_ID.lock().unwrap();
    let msg = format!(r#"{{"id": {}, {}}}"#, *id, msg);
    wsclient.send_message(&Message::text(&msg)).unwrap();
    *id += 1;
    println!("Sent : {}", msg);
}

fn wait_msg(wsclient: &mut Client<std::net::TcpStream>) -> Option<MsgType> {
    loop {
        let resp = wsclient.recv_message();
        match resp {
            Err(websocket::WebSocketError::NoDataAvailable) => return None,
            _ => {}
        }
        let resp = resp.unwrap();
        if let OwnedMessage::Text(i) = resp {
            let v = serde_json::from_str::<Response>(&i);
            if let Ok(_) = v {
                println!("Resp: {}", i);
                return Some(MsgType::Response);
            } else if let Ok(e) = serde_json::from_str::<Event>(&i) {
                println!("Event: {}", i);
                if e.method == "Debugger.paused" {
                    *LAST_PAUSE.lock().unwrap() = Some(e);
                    return Some(MsgType::PauseEvent);
                }
                return Some(MsgType::OtherEvent);
            }
        } else if let OwnedMessage::Close(_) = resp {
            return None;
        } else {
            println!("Unexpected kind of message : {:?}", resp);
        }
    }
}

fn main() {
    // TODO: better message handle
    let addr = std::env::args().nth(1).unwrap();

    let mut client = ClientBuilder::new(&format!("ws://{}", addr))
        .unwrap()
        .connect_insecure()
        .unwrap();

    let init_msgs = [
        r#""method":"Runtime.enable""#,
        r#""method":"Debugger.enable""#,
        r#""method":"Runtime.runIfWaitingForDebugger""#,
    ];

    for msg in init_msgs.iter() {
        send_msg(msg, &mut client);
        while wait_msg(&mut client).unwrap() != MsgType::Response {}
    }

    //for i in 1..=9 {
    //    send_msg(
    //        &format!(
    //            r#""method":"Debugger.setBreakpointByUrl", "params":{{"lineNumber": {}, "urlRegex": "test.py"}}"#,
    //            i
    //        ),
    //        &mut client,
    //    );
    //    while wait_msg(&mut client).unwrap() != MsgType::Response {}
    //}

    // go to test.py
    loop {
        send_msg(r#""method":"Debugger.stepOver""#, &mut client);
        while wait_msg(&mut client).unwrap() != MsgType::PauseEvent {}
        let evn = LAST_PAUSE.lock().unwrap().clone().unwrap();
        if evn.params["callFrames"].as_array().unwrap()[0]["url"]
            .as_str()
            .unwrap()
            .contains("test.py")
        {
            break;
        }
    }

    {
        let mut f = std::fs::File::create("trace.txt").unwrap();
        use std::io::Write;
        write!(f, "{:?}", (*LAST_PAUSE.lock().unwrap()).clone().unwrap());
        'out: loop {
            send_msg(r#""method":"Debugger.stepOver""#, &mut client);
            loop {
                match wait_msg(&mut client) {
                    Some(MsgType::PauseEvent) => {
                        break;
                    }
                    Some(_) => {}
                    None => {
                        break 'out;
                    }
                }
            }
            writeln!(f, "{:?}", (*LAST_PAUSE.lock().unwrap()).clone().unwrap());
        }
    }
}

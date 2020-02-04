#[macro_use]
extern crate lazy_static;

use serde::Deserialize;
use serde::Serialize;

mod chrome_dev_types;
use chrome_dev_types::*;

#[derive(Clone, Debug, PartialEq)]
pub enum MsgType {
    Response(Response),
    PauseEvent(Event),
    OtherEvent(Event),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Step {
    loc: Location,
    vars: std::collections::BTreeMap<String, Vec<PropertyDescriptor>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Trace {
    src: Vec<String>,
    steps: Vec<Step>,
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
        if let Err(websocket::WebSocketError::NoDataAvailable) = resp {
            return None;
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
            if let MsgType::Response(_) = wait_msg(&mut c).unwrap() {
                break;
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
            if let MsgType::Response(_) = wait_msg(&mut c).unwrap() {
                break;
            }
        }
    }
}

fn jump_to_file(filename: &str, mut c: &mut Client) -> Event {
    loop {
        let evn;
        send_msg(r#""method":"Debugger.stepOver""#, &mut c);
        loop {
            if let MsgType::PauseEvent(e) = wait_msg(&mut c).unwrap() {
                evn = e;
                break;
            }
        }
        if evn.try_get_callframes().unwrap()[0].url.contains(filename) {
            return evn;
        }
    }
}

fn runtime_get_properties(id: String, mut c: &mut Client) -> Option<Vec<PropertyDescriptor>> {
    send_msg(
        &format!(
            r#""method":"Runtime.getProperties", params: {{"objectId": {}}}"#,
            id
        ),
        &mut c,
    );
    match wait_msg(&mut c) {
        Some(MsgType::Response(e)) => Some(
            serde_json::from_value(e.result["result"].clone())
                .expect("result should be PropertyDescriptor in this context"),
        ),
        _ => None,
    }
}

fn main() {
    // TODO: better message handling
    // e.g. use queue for event handling

    let filename = std::env::args().nth(1);
    if filename.is_none() {
        eprintln!("Error: file name not provided");
        std::process::exit(1);
    }
    let filename = &filename.unwrap();

    let graal_exec = std::env::var("GRAAL_EXECUTABLE");
    if graal_exec.is_err() {
        eprintln!("Error: $GRAAL_EXECUTABLE is not set");
        std::process::exit(1);
    }
    let graal_exec = graal_exec.unwrap();

    let log_stdout = std::fs::File::create("graal.log").unwrap();
    let log_stderr = log_stdout.try_clone().unwrap();
    let mut svr = std::process::Command::new(graal_exec)
        .args(&[
            "--log.level=ALL",
            "--inspect.Path=graal-inspector",
            "--inspect",
            filename,
        ])
        .stdout(log_stdout)
        .stderr(log_stderr)
        .spawn()
        .expect("GRAAL EXECUTION FAILURE");

    let addr = "ws://127.0.0.1:9229/graal-inspector";

    let mut c = {
        let mut res;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            res = websocket::ClientBuilder::new(addr)
                .unwrap()
                .connect_insecure();
            if res.is_ok() {
                break;
            }
            eprintln!("Warn: connection failed, retrying...");
        }
        res.unwrap()
    };

    init_debugger(&mut c);

    let src = std::fs::read_to_string(filename)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    set_breakpoint_all(src.len(), filename, &mut c);

    let mut evn = jump_to_file("test.py", &mut c);

    {
        let mut trace = Trace {
            src,
            steps: Default::default(),
        };
        let mut f = std::fs::File::create("trace.txt").unwrap();
        use std::io::Write;
        'out: loop {
            let cfs = evn.try_get_callframes().unwrap();
            let loc = cfs[0].location.clone();
            let mut step = Step {
                loc,
                vars: Default::default(),
            };
            //println!("cf size: {}", cfs.len());
            for cf in cfs {
                for sc in cf.scope_chain {
                    if let Some(ref i) = sc.name {
                        let exc = ["builtins", "__main__", "<module"];
                        if exc.iter().any(|s| i.starts_with(s)) {
                            continue;
                        }
                    }
                    let var_array = runtime_get_properties(sc.object.object_id.unwrap(), &mut c);
                    if let Some(vs) = var_array {
                        //FIXME: wait until __main__ appears (to exclude function
                        // definitions)
                        step.vars.entry(sc.name.unwrap()).or_insert_with(|| {
                            vs.into_iter()
                                .filter(|pd| {
                                    if let Some(ref i) = pd.value {
                                        i.r#type != "function" && !pd.name.starts_with("__")
                                    } else {
                                        false
                                    }
                                })
                                .collect::<Vec<_>>()
                        });
                    } else {
                        break 'out;
                    }
                }
            }
            trace.steps.push(step);

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
        writeln!(f, "{}", serde_json::to_string(&trace).unwrap()).unwrap();
    }
    let _ = svr.kill();
}

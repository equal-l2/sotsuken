use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Response {
    pub result: Value,
    pub id: isize,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Event {
    pub method: String,
    pub params: Value,
}

impl Event {
    pub fn try_get_callframes(&self) -> Option<Vec<CallFrame>> {
        serde_json::from_value::<Vec<CallFrame>>(self.params["callFrames"].clone()).ok()
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CallFrame {
    pub call_frame_id: String,
    pub function_name: String,
    pub location: Location,
    pub scope_chain: Vec<Scope>,
    pub url: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub column_number: usize,
    pub line_number: usize,
    pub script_id: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub name: Option<String>,
    pub object: RemoteObject,
    pub r#type: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteObject {
    pub r#type: String,
    pub object_id: Option<String>,
    pub value: Option<Value>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct PropertyDescriptor {
    pub name: String,
    pub value: RemoteObject,
}

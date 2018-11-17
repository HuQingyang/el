
use web_view::{WebView, MyUnique};
use installer;

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
#[serde(tag = "cmd")]
enum Status {
    ok,
    error,
    init,
    download,
    unzip,
    install,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
#[serde(tag = "cmd")]
pub enum Cmd {
    init,
    log { text: String },
}

#[derive(Deserialize)]
pub struct StateItem {}

pub fn exec_callback<'a, T>(webview: &mut WebView<'a, T>, arg: &str, state: &mut Vec<StateItem>) {
    match serde_json::from_str(arg).unwrap() {
        Cmd::log { text } => println!("{}", text),
        Cmd::init => installer::install(webview)
    }
}

pub fn dispatch_to_render<'a, T>(event: &str, arg: &str, webview: &mut WebView<'a, T>) {
    let code = format!("window.rpc.dispatch('{}', {})", event, arg);
    webview.eval(&code);
}

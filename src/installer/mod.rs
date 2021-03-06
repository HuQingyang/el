
use unzip;
use serde_json;
use utils;
use helper;
use downloader;
use rpc;
use web_view::{WebView, Content, WebViewBuilder};
use model::{Version};
use std::path::{PathBuf, Path};
use std::fs;
use std::process::Command;
use model;
use statics;
use timer;
use chrono;
use std::sync::mpsc::channel;


pub fn open_install_helper() {
    let view = ::std::thread::spawn(move || {
        println!("111");

        let title = "Electron Platform";
        let html = utils::generate_html(
            vec![],
            vec![
                include_str!("../view/js/rpc.js"),
                include_str!("../view/js/main.js"),
            ],
        );

        let size = (800, 480);
        let resizable = true;
        let debug = true;
        let state: Vec<rpc::StateItem> = vec![];

        WebViewBuilder::new()
            .title(title)
            .content(Content::Html(html))
            .size(size.0, size.1)
            .resizable(true)
            .debug(true)
            .user_data(state)
            .invoke_handler(|_, _| { Ok(()) })
            .build()
            .expect("Build Error")
            .run()
            .expect("Run Error");
    });

    view.join().expect("Join Error");
}

pub fn install<T>(webview: &mut WebView<T>) {
    // TODO: Update render state
    rpc::dispatch(
        "stateChange",
        "{ state: 'init' }",
        webview
    );

    let config = &statics::CONFIG;
    match downloader::get_valid_runtime_version(&config.runtime)  {
        Err(_) => {
            return rpc::dispatch(
                "stateChange",
                "{ state: 'error', error: 'Get valid runtime version failed.' }",
                webview
            );
        },
        Ok(version) => {
            println!("{:?}", &version);
            rpc::dispatch(
                "stateChange",
                &format!(
                    "{{ state: 'download', version: {} }}",
                    helper::version_to_string(&version)
                ),
                webview
            );
        }
    }
    return;
    match downloader::get_valid_runtime_version(&config.runtime)  {
        Err(_) => {
            return rpc::dispatch(
                "stateChange",
                "{ state: 'error', error: 'Get valid runtime version failed.' }",
                webview
            );
        },
        Ok(version) => {
            rpc::dispatch(
                "stateChange",
                &format!(
                    "{{ state: 'download', version: {} }}",
                    helper::version_to_string(&version)
                ),
                webview
            );
            match downloader::download_runtime(&version) {
                None => {
                    return rpc::dispatch(
                        "stateChange",
                        "{ state: 'error', error: 'Download runtime failed.' }",
                        webview
                    );
                }
                Some(v) => {
                    rpc::dispatch(
                        "stateChange",
                        "{ state: 'unzip' }",
                        webview
                    );
                    match unzip_runtime(&v) {
                        Err(why) => {
                            return rpc::dispatch(
                                "stateChange",
                                "{ state: 'error', error: 'Unzip runtime failed.' }",
                                webview
                            );
                        },
                        Ok(unzip_path) => {
                            rpc::dispatch(
                                "stateChange",
                                "{ state: 'install' }",
                                webview
                            );
                            install_runtime(unzip_path, &v);
                        }
                    }
                }
            }
        }
    }
}

pub fn install_runtime(unzip_path: PathBuf, v: &Version) -> Result<(), String> {
    let runtime_path = unzip_path.join("Electron.app/Contents/Frameworks");
    let target_path = helper::get_runtimes_path()
        .join(helper::version_to_string(v));
    let move_result = fs::rename(runtime_path, target_path);
    fs::remove_dir_all(&unzip_path);
    if let Ok(_) = move_result {
        Ok(())
    } else {
        Err("Move runtime files failed".to_owned())
    }
}

pub fn unzip_runtime(v: &Version) -> Result<PathBuf, String> {
    let from = helper::get_platform_path()
        .join(format!("temp/{}.zip", helper::version_to_string(v)));
    let to = helper::get_platform_path()
        .join(format!("temp/{}", helper::version_to_string(v)));
    unzip_file(&from, to)
}

fn unzip_file(file_path: &PathBuf, to: PathBuf) -> Result<PathBuf, String> {
    if cfg!(target_os = "macos") {
        let result = Command::new("unzip")
            .args(&[
                "-n",
                file_path.to_str().unwrap(),
                "-d",
                to.to_str().unwrap()
            ])
            .output();
        match result {
            Ok(_) => Ok(to),
            Err(_) => Err("Failed to unzip file".to_owned())
        }
    } else {
        let file = fs::File::open(file_path).unwrap();
        let archive_result = unzip::Unzipper::new(file, &to).unzip();
        match archive_result {
            Ok(_) => Ok(to),
            Err(_) => Err("Failed to unzip file".to_owned())
        }
    }
}

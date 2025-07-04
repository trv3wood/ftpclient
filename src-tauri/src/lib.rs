use std::{path::PathBuf, sync::Mutex};

use tauri::State;

mod client;
mod message;

use client::Client;
#[macro_export]
macro_rules! mydbg {
    ($($val:expr),+ $(,)?) => {
        if cfg!(debug_assertions) {
            dbg!($($val),+)
        } else {
            ($($val),+)
        }
    };
    ($val:expr $(,)?) => {
        if cfg!(debug_assertions) {
            dbg!($val)
        } else {
            $val
        }
    };
    () => {
        dbg!()
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("解析字符串失败: '{0}'")]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseAddr(#[from] std::net::AddrParseError),
    #[error("{0}")]
    Server(String),
}
#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    Io(String),
    Utf8(String),
    AddrParse(String),
    ParseInt(String),
    Server(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::Io(_) => ErrorKind::Io(error_message),
            Self::Utf8(_) => ErrorKind::Utf8(error_message),
            Self::ParseAddr(_) => ErrorKind::AddrParse(error_message),
            Self::Server(_) => ErrorKind::Server(error_message),
            Self::ParseInt(_) => ErrorKind::ParseInt(error_message),
        };
        error_kind.serialize(serializer)
    }
}

#[tauri::command]
fn login(
    host: String,
    name: String,
    passwd: String,
    port: u16,
    state: State<'_, Mutex<Client>>,
) -> Result<(), Error> {
    mydbg!((&host, &port, &name, &passwd));
    let mut client = Client::build(host, name, passwd, port)?;
    let mut state = state.lock().unwrap();
    client.login()?;
    *state = client;
    Ok(())
}

#[tauri::command]
fn logout(state: State<'_, Mutex<Client>>) -> Result<(), Error> {
    let mut state = state.lock().unwrap();
    let _ = state.send_command("QUIT");
    Ok(())
}

#[tauri::command]
async fn nls(state: State<'_, Mutex<Client>>, path: &str) -> Result<Vec<String>, Error> {
    let mut client = state.lock().unwrap();
    client.nlst(path)
}

#[tauri::command]
async fn pwd(state: State<'_, Mutex<Client>>) -> Result<String, Error> {
    let mut client = state.lock().unwrap();
    client.pwd()
}

#[tauri::command]
async fn download(state: State<'_, Mutex<Client>>, file: &str) -> Result<PathBuf, Error> {
    let mut client = state.lock().unwrap();
    client.download(file)
}

#[tauri::command]
async fn quit(state: State<'_, Mutex<Client>>) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.quit()
}

#[tauri::command]
async fn cd(state: State<'_, Mutex<Client>>, path: &str) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.cd(&path)
}
#[tauri::command]
async fn upload(state: State<'_, Mutex<Client>>, file: &str) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.upload(file)
}

#[tauri::command]
async fn rm(state: State<'_, Mutex<Client>>, path: &str) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.rm(&path)
}

#[tauri::command]
async fn mkdir(state: State<'_, Mutex<Client>>, path: &str) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.mkdir(&path)
}

#[tauri::command]
async fn rmdir(state: State<'_, Mutex<Client>>, path: &str) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.rmdir(path)
}

#[tauri::command]
async fn rename(
    state: State<'_, Mutex<Client>>,
    old_path: &str,
    new_path: &str,
) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    client.rename(old_path, new_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            login, logout, nls, pwd, download, quit, cd, upload, rm, mkdir, rmdir, rename
        ])
        .manage(Mutex::new(Client::default()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

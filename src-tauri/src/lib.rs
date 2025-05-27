use std::{
    io::{Read, Write},
    sync::Mutex,
};

use tauri::State;

mod client;
use client::Client;

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
    #[error("服务端错误: {0}")]
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
    println!("login {host}:{port} {name} {passwd} ");
    let client = Client::build(host, name, passwd, port)?;
    let mut state = state.lock().unwrap();
    *state = client;
    Ok(())
}

#[tauri::command]
fn logout(state: State<'_, Mutex<Client>>) -> Result<(), Error> {
    let mut state = state.lock().unwrap();
    let _ = state.send_command("QUIT");
    Ok(())
}

fn is_data_conn_open(res: &[u8]) -> bool {
    res.starts_with(b"150") || res.starts_with(b"226")
}
#[tauri::command]
async fn nls(state: State<'_, Mutex<Client>>, path: &str) -> Result<Vec<String>, Error> {
    let mut client = state.lock().unwrap();
    // 创建数据连接
    let mut data_socket = client.pasv()?;
    // 发送 NLST 命令获取目录列表
    let response = client.send_command(&format!("NLST {}", path))?;
    if !is_data_conn_open(response) {
        return server_error!("目录列表获取失败");
    }
    let buffer = client.buffer_mut()?;
    let bytes = data_socket.read(buffer)?;
    if bytes == 0 {
        return server_error!("目录列表为空或读取失败");
    }
    let data = String::from_utf8_lossy(&buffer[..bytes]);
    let data = data.split('\n')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();
    Ok(data)
}
#[tauri::command]
fn is_logged_in(state: State<'_, Mutex<Client>>) -> bool {
    let client = state.lock().unwrap();
    client.is_logged_in()
}

#[tauri::command]
async fn pwd(state: State<'_, Mutex<Client>>) -> Result<String, Error> {
    let mut client = state.lock().unwrap();
    client.pwd()
}

#[tauri::command]
async fn download(file: String, state: State<'_, Mutex<Client>>) -> Result<(), Error> {
    let mut client = state.lock().unwrap();
    let mut data_socket = client.pasv()?;

    let response = client.send_command(&format!("RETR {}", file))?;
    if !is_data_conn_open(response) {
        return server_error!("文件下载失败");
    }
    data_socket.set_nonblocking(true)?;
    let download_dir = dirs::download_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "无法获取下载目录",
    ))?;
    let file_path = download_dir.join(&file);
    let mut file = std::fs::File::create(&file_path)?;
    let buffer = client.buffer_mut()?;
    loop {
        match data_socket.read(buffer) {
            Ok(0) => break, // 连接关闭
            Ok(n) => file.write_all(&buffer[..n])?,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue, // 非阻塞模式下继续读取
            Err(e) => return Err(Error::Io(e)),
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login, logout, nls, pwd, download, is_logged_in])
        .manage(Mutex::new(Client::new()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

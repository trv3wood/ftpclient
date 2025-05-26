use std::{
    io::{Read, Write}, net::{IpAddr, SocketAddr, TcpStream}, str::FromStr, sync::Mutex, time::Duration
};

use tauri::State;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("解析字符串失败: '{0}'")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("错误的IP地址: '{0}'")]
    InvalidIpAddr(String),
    #[error("服务端错误: {0}")]
    Server(String),
}
macro_rules! server_error {
    ($msg:expr) => {
        Err(Error::Server($msg.into()))
    };
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    Io(String),
    Utf8(String),
    InvalidIpAddr(String),
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
            Self::InvalidIpAddr(_) => ErrorKind::InvalidIpAddr(error_message),
            Self::Server(_) => ErrorKind::Server(error_message),
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
    state: State<'_, Mutex<Option<Client>>>,
) -> Result<(), Error> {
    println!("login {host}:{port} {name} {passwd} ");
    let mut client = Client::build(host, name, passwd, port)?;
    let username_res = client.send_command(&format!("USER {}", client.name))?;
    if !username_res.starts_with(b"331") {
        dbg!(String::from_utf8_lossy(username_res));
        return server_error!("用户名错误或未找到");
    }
    let pass_res = client.send_command(&format!("PASS {}", client.passwd))?;
    if !pass_res.starts_with(b"230") {
        dbg!(String::from_utf8_lossy(pass_res));
        return server_error!("密码错误或未找到");
    }
    let mut state = state.lock().unwrap();
    state.take(); // 清除之前的客户端状态
    *state = Some(client);
    Ok(())
}

#[tauri::command]
fn logout(state: State<'_, Mutex<Option<Client>>>) -> Result<(), Error> {
    let mut state = state.lock().unwrap();
    let _ = state.take().unwrap().send_command("QUIT");
    Ok(())
}

#[tauri::command]
async fn dir_list(state: State<'_, Mutex<Option<Client>>>) -> Result<String, Error> {
    let mut state = state.lock().unwrap();
    let client = state.as_mut().ok_or_else(|| Error::Server("未登录".into()))?;
    let response = client.send_command("LIST")?;
    if !response.starts_with(b"150") && !response.starts_with(b"226") {
        return server_error!("目录列表获取失败");
    }
    Ok(String::from_utf8_lossy(response).to_string())
}

#[tauri::command]
async fn pwd(state: State<'_, Mutex<Option<Client>>>) -> Result<String, Error> {
    let mut state = state.lock().unwrap();
    let client = state.as_mut().ok_or_else(|| Error::Server("未登录".into()))?;
    let response = client.send_command("PWD")?;
    if !response.starts_with(b"257") {
        return server_error!("当前工作目录获取失败");
    }
    // 提取目录路径
    let path = String::from_utf8_lossy(&response[4..]).trim_matches('"').to_string();
    Ok(path)
}

#[tauri::command]
async fn download(
    file: String,
    state: State<'_, Mutex<Option<Client>>>,
) -> Result<(), Error> {
    let mut state = state.lock().unwrap();
    let client = state.as_mut().ok_or_else(|| Error::Server("未登录".into()))?;
    let pasv_response = client.send_command("PASV")?;
    if !pasv_response.starts_with(b"227") {
        return server_error!("进入被动模式失败");
    }
    let data_socket_info = String::from_utf8_lossy(pasv_response);
    println!("PASV Response: {}", data_socket_info);

    // let response = client.send_command(&format!("RETR {}", file))?;
    // if !response.starts_with(b"150") && !response.starts_with(b"226") {
    //     return server_error!("文件下载失败");
    // }
    // 假设下载成功，返回文件名
    Ok(())
}

#[derive(Debug)]
struct Client {
    name: String,
    passwd: String,
    socket: TcpStream,
    buffer: Box<[u8; 1024]>, // 使用 Box<[u8]> 代替 Vec<u8>，更适合只读操作
    // data_socket: Option<TcpStream>,
}

macro_rules! slice2string {
    ($vec:expr) => {
        String::from_utf8_lossy($vec).to_string()
    };
    () => {
        
    };
}
impl Client {
    pub fn build(host: String, name: String, passwd: String, port: u16) -> Result<Self, Error> {
        let ip_addr = IpAddr::from_str(&host);
        if let Err(_) = ip_addr {
            return Err(Error::InvalidIpAddr(host));
        }
        let ip_addr = ip_addr.unwrap();
        let socket_addr = SocketAddr::new(ip_addr, port);
        let mut socket = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
        let mut buffer = Box::new([0; 1024]);
        let read = socket.read(buffer.as_mut())?;
        if read == 0 {
            return server_error!("连接到服务器失败，可能是服务器未启动或地址错误");
        }
        if !buffer.starts_with(b"220") {
            return server_error!(slice2string!(&buffer[0..read]));
        }
        Ok(Self {
            name,
            passwd,
            socket,
            buffer,
            // data_socket: None,
        })
    }
    pub fn send_command(&mut self, command: &str) -> Result<&[u8], Error> {
        self.socket.write_all(dbg!(command).as_bytes())?;
        self.socket.write_all(b"\r\n")?;
        self.socket.flush()?;
        let bytes_read = self.socket.read(self.buffer.as_mut())?;
        if bytes_read == 0 {
            return server_error!("服务器没有响应");
        }
        Ok(&self.buffer[0..bytes_read])
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login, logout, dir_list, pwd, download])
        .manage(Mutex::new(None::<Client>))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

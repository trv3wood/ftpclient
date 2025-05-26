use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    str::FromStr,
    sync::Mutex,
    time::Duration,
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
    #[error("客户端错误: {0}")]
    ClientError(String),
    #[error("服务端错误: {0}")]
    ServerError(String),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    Io(String),
    Utf8(String),
    InvalidIpAddr(String),
    ClientError(String),
    ServerError(String),
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
            Self::ClientError(_) => ErrorKind::ClientError(error_message),
            Self::ServerError(_) => ErrorKind::ServerError(error_message),
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
    client: State<'_, Mutex<Client>>,
) -> Result<(), Error> {
    println!("login {host}:{port} {name} {passwd} ");
    let ip_addr = IpAddr::from_str(&host);
    if let Err(_) = ip_addr {
        return Err(Error::InvalidIpAddr(host));
    }
    let ip_addr = ip_addr.unwrap();
    let socket_addr = SocketAddr::new(ip_addr, port);
    let mut connection = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
    let mut client = client
        .lock()
        .map_err(|e| Error::ClientError(e.to_string()))?;
    client.host = host;
    client.name = name;
    client.passwd = passwd;
    client.port = port;
    let mut buffer = [0; 1024];
    let bytes_read = connection.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    if response.starts_with("220") {
        dbg!(&response);
    } else {
        return Err(Error::ServerError(format!(
            "Unexpected response: {}",
            response
        )));
    }
    connection.write_all(b"USER ftp\r\n")?;
    connection.flush()?;
    // connection.write_all(b"PASS passwd\r\n")?;
    // connection.flush()?;
    // 读取服务器响应
    let bytes_read = connection.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    // 检查是否为 331 响应
    if response.starts_with("331") {
        // 发送 PASS 命令
        connection.write_all(b"PASS passwd\r\n")?;
        connection.flush()?;
    } else {
        return Err(Error::ServerError(format!(
            "Unexpected response: {}",
            response
        )));
    }
    client.socket = Some(connection);
    dbg!(&client);
    Ok(())
}

#[derive(Debug)]
struct Client {
    host: String,
    name: String,
    passwd: String,
    port: u16,
    socket: Option<TcpStream>,
    // data_socket: Option<TcpStream>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            host: "".into(),
            name: "".into(),
            passwd: "".into(),
            port: 0,
            socket: None,
            // data_socket: None,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login])
        .manage(Mutex::new(Client::new()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[macro_export]
macro_rules! server_error {
    ($msg:expr) => {
        Err(Error::Server($msg.into()))
    };
}

use std::{
    borrow::Cow,
    io::{BufReader, Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use crate::{message::*, mydbg, Error};

#[derive(Debug)]
pub struct Client {
    name: String,
    passwd: String,
    socket: Option<TcpStream>,
    buf: Box<[u8; 1024]>,
}
impl Default for Client {
    fn default() -> Self {
        Self {
            name: "".into(),
            passwd: "".into(),
            socket: None,
            buf: Box::new([0; 1024]),
        }
    }
}

fn is_data_conn_open(res: impl AsRef<str>) -> bool {
    res.as_ref().starts_with("150") || res.as_ref().starts_with(CLOSING_DATA_CONNECTION)
}

impl Client {
    pub fn build(host: String, name: String, passwd: String, port: u16) -> Result<Self, Error> {
        let ip_addr = IpAddr::from_str(&host)?;
        let socket_addr = SocketAddr::new(ip_addr, port);
        let mut socket = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
        let mut buffer = Box::new([0; 1024]);
        let read = socket.read(buffer.as_mut())?;
        if read == 0 {
            return server_error!("连接到服务器失败，可能是服务器未启动或地址错误");
        }
        if !buffer.starts_with(b"220") {
            return server_error!(String::from_utf8_lossy(&buffer[0..read]));
        }
        Ok(Self {
            name,
            passwd,
            socket: Some(socket),
            buf: Box::new([0; 1024]),
        })
    }
    pub fn send_command(&mut self, command: impl AsRef<[u8]>) -> Result<Cow<str>, Error> {
        let mut sock = self
            .socket
            .take()
            .ok_or_else(|| Error::Server("未连接到服务器，请先登录".into()))?;
        sock.write_all(mydbg!(command.as_ref()))?;
        sock.flush()?;
        let bytes_read = sock.read(self.buf.as_mut())?;
        if bytes_read == 0 {
            return server_error!("服务器没有响应");
        }
        self.socket = Some(sock);
        let response = &self.buf[0..bytes_read];
        let response = String::from_utf8_lossy(response);
        mydbg!(&response);
        Ok(response)
    }
    pub fn login(&mut self) -> Result<(), Error> {
        self.send_command(&format!("USER {}", self.name))?;
        let response = self.send_command(&format!("PASS {}\r\n", self.passwd))?;
        if response.starts_with(USER_LOGGED_IN) {
            Ok(())
        } else if response.starts_with(NOT_LOGGED_IN) {
            server_error!("登录失败，用户名或密码错误")
        } else {
            Err(Error::Server(format!("登录失败，服务器响应: {}", response)))
        }
    }
    pub fn pasv(&mut self) -> Result<TcpStream, Error> {
        let data_socket_info = self.send_command("PASV")?;
        if !data_socket_info.starts_with(ENTERING_PASSIVE_MODE) {
            return server_error!(&format!("进入被动模式失败 {}", data_socket_info));
        }
        mydbg!(&data_socket_info);
        // 解析 PASV 响应以获取数据连接信息
        let parts: Vec<&str> = data_socket_info.split(|c| c == '(' || c == ')').collect();
        if parts.len() < 2 {
            return server_error!("PASV 响应格式错误");
        }
        let addr_parts: Vec<&str> = parts[1].split(',').collect();
        if addr_parts.len() < 6 {
            return server_error!("PASV 响应地址部分格式错误");
        }
        let ip = addr_parts[0..4].join(".");
        let port = addr_parts[4].parse::<u16>()? * 256 + addr_parts[5].parse::<u16>()?;
        let data_socket_addr = SocketAddr::new(IpAddr::from_str(&ip)?, port);
        mydbg!(data_socket_addr);
        let data_socket = TcpStream::connect_timeout(&data_socket_addr, Duration::from_secs(5))?;
        Ok(data_socket)
    }
    pub fn pwd(&mut self) -> Result<String, Error> {
        let response = self.send_command("PWD")?;
        if !response.starts_with(PATHNAME_CREATED) {
            return server_error!("获取当前工作目录失败");
        }
        // 提取目录路径
        let path = response[4..].trim_matches('"').to_string();
        Ok(path)
    }

    pub fn nlst(&mut self, path: &str) -> Result<Vec<String>, Error> {
        // // 创建数据连接
        let mut data_socket = self.pasv()?;
        // // 发送 NLST 命令获取目录列表
        let response = self.send_command(&format!("NLST {}", path))?;
        if !is_data_conn_open(response) {
            return server_error!("目录列表获取失败");
        }
        let buf = self.buf.as_mut();
        let bytes = data_socket.read(buf)?;
        if bytes == 0 {
            return Ok(Vec::new()); // 如果没有数据，返回空列表
        }
        let data = String::from_utf8_lossy(&buf[..bytes]);
        let data = data
            .split('\n')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();
        let transfer_response = self.read()?;
        if !transfer_response.starts_with(CLOSING_DATA_CONNECTION) {
            self.socket.take();
            return server_error!("数据传输结束时服务器响应错误");
        }
        Ok(data)
    }
    fn read(&mut self) -> Result<Cow<str>, Error> {
        let mut sock = self
            .socket
            .take()
            .ok_or_else(|| Error::Server("未连接到服务器，请先登录".into()))?;
        let bytes_read = sock.read(self.buf.as_mut())?;
        if bytes_read == 0 {
            return server_error!("服务器没有响应");
        }
        self.socket = Some(sock);
        Ok(String::from_utf8_lossy(&self.buf[0..bytes_read]))
    }
    pub fn download(&mut self, file: &str) -> Result<PathBuf, Error> {
        let mut data_socket = self.pasv()?;

        let response = self.send_command(&format!("RETR {}", file))?;
        if !is_data_conn_open(&response) {
            return Err(Error::Server(response[4..].to_string()));
        }
        data_socket.set_nonblocking(true)?;
        let download_dir = dirs::download_dir().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "无法获取下载目录",
        ))?;
        let file_path = download_dir.join(&file);
        let mut file = std::fs::File::create(&file_path)?;
        let buffer = self.buf.as_mut();
        loop {
            match data_socket.read(buffer) {
                Ok(0) => break, // 连接关闭
                Ok(n) => file.write_all(&buffer[..n])?,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue, // 非阻塞模式下继续读取
                Err(e) => return Err(Error::Io(e)),
            }
        }
        let transfer_response = self.read()?;
        if !transfer_response.starts_with(CLOSING_DATA_CONNECTION) {
            self.socket.take();
            return server_error!("数据传输结束时服务器响应错误");
        }
        Ok(file_path)
    }
    pub fn upload(&mut self, file: &str) -> Result<(), Error> {
        let mut data_socket: TcpStream = self.pasv()?;
        let path = Path::new(file);
        let filename = path.file_name().unwrap().to_string_lossy();
        let response = self.send_command(&format!("STOR {}", mydbg!(filename)))?;
        if !is_data_conn_open(&response) {
            return server_error!("上传文件失败");
        }
        let mut bufreader = BufReader::new(std::fs::File::open(&path)?);
        std::io::copy(&mut bufreader, &mut data_socket)?;
        // 关闭数据连接
        data_socket.shutdown(std::net::Shutdown::Both)?;

        // 读取服务器响应以确认传输结束
        let transfer_response = self.read()?;
        if !transfer_response.starts_with(CLOSING_DATA_CONNECTION) {
            self.socket.take();
            return server_error!("数据传输结束时服务器响应错误");
        }
        Ok(())
    }
    pub fn quit(&mut self) -> Result<(), Error> {
        let response = self.send_command("QUIT")?;
        if !response.starts_with(SERVICE_CLOSING_CONTROL_CONNECTION) {
            self.socket.take();
            return server_error!("退出失败");
        }
        self.socket.take();
        Ok(())
    }
    pub fn cd(&mut self, path: &str) -> Result<(), Error> {
        let response = self.send_command(&format!("CWD {}", path))?;
        if !response.starts_with(FILE_ACTION_COMPLETED) {
            return server_error!(&response[4..].to_string());
        }
        Ok(())
    }
    pub fn rm(&mut self, path: &str) -> Result<(), Error> {
        let response = self.send_command(&format!("DELE {}", path))?;
        if !response.starts_with(FILE_ACTION_COMPLETED) {
            return server_error!(&response[4..].to_string());
        }
        Ok(())
    }
    pub fn mkdir(&mut self, path: &str) -> Result<(), Error> {
        let response = self.send_command(&format!("MKD {}", path))?;
        if !response.starts_with(FILE_ACTION_COMPLETED) {
            return server_error!(&response[4..].to_string());
        }
        Ok(())
    }
    pub fn rmdir(&mut self, path: &str) -> Result<(), Error> {
        let response = self.send_command(&format!("RMD {}", path))?;
        if !response.starts_with(FILE_ACTION_COMPLETED) {
            return server_error!(&response[4..].to_string());
        }
        Ok(())
    }
    pub fn rename(&mut self, old: &str, new: &str) -> Result<(), Error> {
        let response = self.send_command(&format!("RNFR {}", old))?;
        if !response.starts_with(FILE_ACTION_NEEDS_FURTHER_INFO) {
            return server_error!(&response[4..].to_string());
        }
        let response = self.send_command(&format!("RNTO {}", new))?;
        if !response.starts_with(FILE_ACTION_COMPLETED) {
            return server_error!(&response[4..].to_string());
        }
        Ok(())
    }
}

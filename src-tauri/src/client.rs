#[macro_export]
macro_rules! server_error {
    ($msg:expr) => {
        Err(Error::Server($msg.into()))
    };
}

use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    str::FromStr,
    time::Duration,
};

use crate::Error;

pub struct Client(Option<ClientInner>);
impl Client {
    pub fn new() -> Self {
        Self(None)
    }
    fn get_mut(&mut self) -> Result<&mut ClientInner, Error> {
        self.0
            .as_mut()
            .ok_or_else(|| Error::Server("未登录".into()))
    }
    pub fn build(host: String, name: String, passwd: String, port: u16) -> Result<Self, Error> {
        let mut inner = ClientInner::build(host, name, passwd, port)?;
        let username_res = inner.send_command(&format!("USER {}", inner.name))?;
        if !username_res.starts_with(b"331") {
            dbg!(String::from_utf8_lossy(username_res));
            return server_error!("用户名错误或未找到");
        }
        let pass_res = inner.send_command(&format!("PASS {}", inner.passwd))?;
        if !pass_res.starts_with(b"230") {
            dbg!(String::from_utf8_lossy(pass_res));
            return server_error!("密码错误或未找到");
        }
        let pwd_res = inner.pwd()?;
        inner.root = pwd_res.to_string();
        Ok(Self(Some(inner)))
    }
    pub fn buffer_mut(&mut self) -> Result<&mut [u8], Error> {
        Ok(self.get_mut()?.buffer.as_mut())
    }
    pub fn send_command(&mut self, command: &str) -> Result<&[u8], Error> {
        self.get_mut()?.send_command(command)
    }
    pub fn pasv(&mut self) -> Result<TcpStream, Error> {
        self.get_mut()?.pasv()
    }
    pub fn pwd(&mut self) -> Result<String, Error> {
        self.get_mut()?.pwd()
    }
    pub fn is_logged_in(&self) -> bool {
        self.0.is_some()
    }
}

#[derive(Debug)]
struct ClientInner {
    name: String,
    passwd: String,
    socket: TcpStream,
    buffer: Box<[u8; 1024]>, // 使用 Box<[u8]> 代替 Vec<u8>，更适合只读操
    pwd: String,
    root: String, // 根目录
}

impl ClientInner {
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
            socket,
            buffer,
            pwd: String::new(),
            root: String::new(), // 初始化根目录为空
        })
    }
    pub fn send_command(&mut self, command: &str) -> Result<&[u8], Error> {
        self.socket.write_all(dbg!(command).as_bytes())?;
        self.socket.flush()?;
        let bytes_read = self.socket.read(self.buffer.as_mut())?;
        if bytes_read == 0 {
            return server_error!("服务器没有响应");
        }
        Ok(&self.buffer[0..bytes_read])
    }
    pub fn pasv(&mut self) -> Result<TcpStream, Error> {
        let response = self.send_command("PASV")?;
        if !response.starts_with(b"227") {
            return server_error!("进入被动模式失败");
        }
        let data_socket_info = String::from_utf8_lossy(response);
        println!("PASV Response: {}", data_socket_info);
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
        println!("Data socket address: {}", data_socket_addr);
        let data_socket = TcpStream::connect_timeout(&data_socket_addr, Duration::from_secs(5))?;
        Ok(data_socket)
    }
    pub fn pwd(&mut self) -> Result<String, Error> {
        let response = self.send_command("PWD")?;
        if !response.starts_with(b"257") {
            return server_error!("获取当前工作目录失败");
        }
        // 提取目录路径
        let path = String::from_utf8_lossy(&response[4..])
            .trim_matches('"')
            .to_string();
        self.pwd = path;
        Ok(self.pwd.clone())
    }
}

use std::{net::{IpAddr, SocketAddr, TcpStream}, str::FromStr, time::Duration};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn login(host: &str, name: &str, passwd: &str, port: u16) -> String {
    println!("login {host}{name}{passwd}{port}");
    if let Ok(ip_addr) = IpAddr::from_str(host) {
        let socket_addr = SocketAddr::new(ip_addr, port);
        let connection = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5));
        return match connection {
            Ok(_) => "连接成功".into(),
            Err(e) => e.to_string()
        }
    }
    "请输入正确的ip地址".into()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

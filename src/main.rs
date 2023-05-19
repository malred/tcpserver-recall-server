use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;
fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}
fn main() {
    // 绑定端口
    let server = TcpListener::bind(LOCAL).expect("listener failed to bind");
    // 非阻塞
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");
    let mut clients = vec![];
    // 通道
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        // 连接
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));
            thread::spawn(move || {
                loop {
                    let mut buff = vec![0; MSG_SIZE];
                    // 读取信息
                    match socket.read_exact(&mut buff) {
                        Ok(_) => {
                            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                            println!("{}: {:?}", addr, msg);
                            tx.send(msg).expect("failed to send message to rx");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("closing connection with: {}", addr);
                            break;
                        }
                    }
                }
                sleep();
            });
        }
        // 尝试通过通道接收信息
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    // 转换为字节
                    let mut buff = msg.clone().into_bytes();
                    // 调整缓冲区大小
                    buff.resize(MSG_SIZE, 0);
                    // 写入缓冲区
                    client.write_all(&buff).map(|_| client).ok()
                })
                // 缓冲区数据收集到向量
                .collect::<Vec<_>>();
        }
        sleep(); 
    }
}

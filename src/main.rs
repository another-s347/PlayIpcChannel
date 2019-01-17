use ipc_channel::ipc::*;
use futures::*;
use std::io::Error;
use libc;
use std::time::{Instant, Duration};
use std::{thread, time};

pub unsafe fn fork<F: FnOnce()>(child_func: F) -> libc::pid_t {
    match libc::fork() {
        -1 => panic!("Fork failed: {}", Error::last_os_error()),
        0 => {
            child_func();
            libc::exit(0);
        },
        pid => pid,
    }
}

fn main() {
    let person = "123".to_string();
    let (server0, server0_name) = IpcOneShotServer::new().unwrap();
    let (server2, server2_name) = IpcOneShotServer::new().unwrap();
    let child_pid = unsafe { fork(|| {
        let (tx1, rx1): (IpcSender<String>, IpcReceiver<String>) = channel().unwrap();
        let tx0 = IpcSender::connect(server0_name).unwrap();
        tx0.send(tx1).unwrap();
        let tx2: IpcSender<String> = IpcSender::connect(server2_name).unwrap();
        tx2.send( "handshake".to_string()).unwrap();
        // works correctly:
        // loop {
        //    let m=rx1.recv().unwrap();
        //    println!("loop receive:{:?}",&m);
        // }
        //

        // broken pipe
        let mut s = rx1.map_err(|_|()).for_each(|m|{
            println!("tokio receive:{:?}",&m);
            futures::future::ok(())
        });
        tokio::run(s);
    })};
    let (_, tx1): (_, IpcSender<String>) = server0.accept().unwrap();
    tx1.send(person.clone()).unwrap();
    let (rx, received_person): (_, String) = server2.accept().unwrap();
    let ten_millis = time::Duration::from_millis(1);
    for i in 0..1000 {
        tx1.send(person.clone()).unwrap();
        thread::sleep(ten_millis); // a little sleep is needed
    }
}
use libc::{AF_NETLINK, SOCK_RAW, bind, recv, sockaddr_nl, socket};
use std::{mem, sync::mpsc};

use crate::core::{ParseError, PowerEvent, parser::parse_uevent};

mod core;
mod services;
mod utils;

const NETLINK_KOBJECT_UEVENT: i32 = 15;
const UEVENT_BUFFER_SIZE: usize = 8192;

fn main() {
    let (tx, rx) = mpsc::channel::<PowerEvent>();

    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            debug!("event received: {:?}", event);
        }
    });

    unsafe {
        let fd = socket(AF_NETLINK, SOCK_RAW, NETLINK_KOBJECT_UEVENT);
        if fd < 0 {
            error!("Failed to open Netlink socket");
            std::process::exit(1);
        }

        let mut sa: sockaddr_nl = mem::zeroed();
        sa.nl_family = AF_NETLINK as u16;
        sa.nl_groups = 1;

        let bind_result = bind(
            fd,
            &sa as *const _ as *const _,
            mem::size_of_val(&sa) as u32,
        );
        if bind_result < 0 {
            error!("Failed to bind to Netlink socket");
            std::process::exit(1);
        }

        info!("Listening for kernel events...");

        let mut buf = [0u8; UEVENT_BUFFER_SIZE];
        loop {
            let len = recv(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0);
            if len > 0 {
                let data = &buf[..len as usize];
                match parse_uevent(data) {
                    Ok(event) => {
                        if let Err(e) = tx.send(event) {
                            error!("Consumer thread died, dropping event: {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(ParseError::MissingFields) => {
                        // Normal behavior. It was probably a USB mouse uevent, just ignore it.
                    }
                    Err(e) => {
                        warn!("Failed to parse hardware event: {:?}", e);
                    }
                }
            }
        }
    }
}

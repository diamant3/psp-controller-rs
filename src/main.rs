#![no_std]
#![no_main]

const SERVER_ADDRESS: &str = "192.168.4.1";
const SERVER_PORT: u16 = 80;

psp::module!("psp-controller-rs", 0, 1);

fn connect_apctl(index: i32) {
    unsafe {
        let mut state = psp::sys::ApctlState::Disconnected;

        let mut status = psp::sys::sceNetApctlConnect(index);
        if status < 0 {
            psp::dprintln!("sceNetApctlConnect status: {status:#X}");
        }

        loop {
            status = psp::sys::sceNetApctlGetState(&mut state);
            if status < 0 {
                psp::dprintln!("sceNetApctlGetState status: {status:#X}");
                break;
            }

            match state {
                psp::sys::ApctlState::GotIp => {
                    psp::dprintln!("Connected!");
                    break;
                },
                _ => (),
            }

            psp::sys::sceKernelDelayThread(50 * 1000); // 50 ms
        }
    }
}

fn psp_main() {
    psp::enable_home_button();

    unsafe {
        psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetCommon);
        psp::sys::sceUtilityLoadNetModule(psp::sys::NetModule::NetInet);

        // values got from https://github.com/pspdev/pspsdk/blob/master/src/sdk/inethelper.c#L64
        let _ = psp::sys::sceNetInit(0x20000, 0x20, 0x1000, 0x20, 0x1000);
        let _ = psp::sys::sceNetInetInit();
        let _ = psp::sys::sceNetResolverInit();
        let _ = psp::sys::sceNetApctlInit(0x1600, 0x42);

        let _ = psp::sys::sceCtrlSetSamplingCycle(0);
        let _ = psp::sys::sceCtrlSetSamplingMode(psp::sys::CtrlMode::Digital);
        let mut pad_data = psp::sys::SceCtrlData::default();

        let mut server_address = psp::sys::sockaddr {
            sa_len: 16,
            sa_family: 2,
            sa_data: [0; 14],
        };

        let port_bytes = SERVER_PORT.to_be_bytes();
        for (index, byte) in port_bytes.iter().enumerate() {
            server_address.sa_data[index] = *byte;
        }

        let ip_string = SERVER_ADDRESS.split('.');
        for (index, bytes) in ip_string.enumerate() {
            if let Ok(octet) = bytes.parse::<u8>() {
                server_address.sa_data[index + 2] = octet;
            }
        }

        connect_apctl(1);
        loop {
            let socket = psp::sys::sceNetInetSocket(2, 1, 0);
            if socket < 0 { break; }

            let _ = psp::sys::sceCtrlReadBufferPositive(&mut pad_data, 1);

            if !pad_data.buttons.is_empty() {
                let _ = psp::sys::sceNetInetConnect(socket, &server_address, server_address.sa_len as u32);

                if pad_data.buttons.contains(psp::sys::CtrlButtons::CIRCLE) {
                    psp::dprintln!("CIRCLE was pressed!");
                    let on = "GET /on HTTP/1.1\r\nHost: {SERVER_ADDRESS}\r\nConnection: close\r\n\r\n";
                    psp::sys::sceNetInetSend(socket, on.as_ptr() as *const core::ffi::c_void, on.len(), 0);
                }

                if pad_data.buttons.contains(psp::sys::CtrlButtons::CROSS) {
                    psp::dprintln!("CROSS was pressed!");
                    let off = "GET /off HTTP/1.1\r\nHost: {SERVER_ADDRESS}\r\nConnection: close\r\n\r\n";
                    psp::sys::sceNetInetSend(socket, off.as_ptr() as *const core::ffi::c_void, off.len(), 0);
                }
            }

            psp::sys::sceNetInetClose(socket);
        }

        let _ = psp::sys::sceNetApctlTerm();
        let _ = psp::sys::sceNetResolverTerm();
        let _ = psp::sys::sceNetInetTerm();
        let _ = psp::sys::sceNetTerm();

        let _ = psp::sys::sceUtilityUnloadNetModule(psp::sys::NetModule::NetInet);
        let _ = psp::sys::sceUtilityUnloadNetModule(psp::sys::NetModule::NetCommon);
    }
}

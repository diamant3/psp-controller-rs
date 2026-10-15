#![no_std]
#![no_main]

psp::module!("psp-controller-rs", 0, 1);

// fn psp_clear_screen() {
//     unsafe {
//         let mut fb = core::ptr::null_mut();
//         let mut fb_width = 0 as usize;
//         let _ = psp::sys::sceDisplayGetFrameBuf(&mut fb, &mut fb_width, &mut psp::sys::DisplayPixelFormat::Psm8888, psp::sys::DisplaySetBufSync::Immediate);
// 
//         let size = fb_width * 272 * 4;
//         core::ptr::write_bytes(fb.cast::<u8>(), 0, size);
//     }
// }

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
                psp::sys::ApctlState::GotIp => break,
                _ => psp::dprintln!("Connecting..."),
            }

            psp::sys::sceKernelDelayThread(50 * 1000); // 50 ms
        }
    }
}

fn create_socket() -> i32 {
    let s = unsafe {
        let socket = psp::sys::sceNetInetSocket(2, 1, 0);
        if socket < 0 { return -1; }

        let mut server_address = psp::sys::sockaddr {
            sa_len: 16,
            sa_family: 2,
            sa_data: [0; 14],
        };

        let port_bytes = (80 as u16).to_be_bytes();
        server_address.sa_data[0] = port_bytes[0];
        server_address.sa_data[1] = port_bytes[1];

        let ipv4 = [192, 168, 4, 1];
        server_address.sa_data[2] = ipv4[0];
        server_address.sa_data[3] = ipv4[1];
        server_address.sa_data[4] = ipv4[2];
        server_address.sa_data[5] = ipv4[3];

        let _ = psp::sys::sceNetInetConnect(socket, &server_address, 16);

        socket
    };

    s
}

fn psp_main() {
    psp::enable_home_button();

    unsafe {
        // values got from https://github.com/pspdev/pspsdk/blob/master/src/sdk/inethelper.c#L64
        let _ = psp::sys::sceNetInit(0x20000, 0x20, 0x1000, 0x20, 0x1000);
        let _ = psp::sys::sceNetInetInit();
        let _ = psp::sys::sceNetResolverInit();
        let _ = psp::sys::sceNetApctlInit(0x1600, 0x42);

        let _ = psp::sys::sceCtrlSetSamplingCycle(0);
        let _ = psp::sys::sceCtrlSetSamplingMode(psp::sys::CtrlMode::Digital);

        let mut pad_data = psp::sys::SceCtrlData::default();

        connect_apctl(1);
        let socket = create_socket();
        loop {
            let _ = psp::sys::sceCtrlReadBufferPositive(&mut pad_data, 1);
    
            if pad_data.buttons.contains(psp::sys::CtrlButtons::CIRCLE) {
                psp::dprintln!("CIRCLE was pressed!");
                let on = "GET /on HTTP/1.1\r\nHost: 192.168.4.1\r\nConnection: close\r\n\r\n";
                psp::sys::sceNetInetSend(socket, on.as_ptr() as *const core::ffi::c_void, on.len(), 0);
            }

            if pad_data.buttons.contains(psp::sys::CtrlButtons::CROSS) {
                psp::dprintln!("CROSS was pressed");
                let off = "GET /off HTTP/1.1\r\nHost: 192.168.4.1\r\nConnection: close\r\n\r\n";
                psp::sys::sceNetInetSend(socket, off.as_ptr() as *const core::ffi::c_void, off.len(), 0);
            }
        }
    }
}

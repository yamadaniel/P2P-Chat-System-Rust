// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {

    // P2Pらを初期化しているが、本来は推奨されない
    std::thread::spawn(|| {
        p2p_chat_system_rust_lib::run();
    });

}


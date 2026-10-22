mod p2p;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let p2p = tauri::async_runtime::block_on(p2p::start(handle))?;
            app.manage(p2p);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            logged,
            p2p::my_id,
            p2p::send_message

        ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
 

#[tauri::command]
fn logged() -> String {
    format!("絶対助ける！")
}

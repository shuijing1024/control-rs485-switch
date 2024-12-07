use crate::prelude::CustomAppResult;
use tauri::async_runtime::Mutex;
use tauri::State;

mod control;
mod prelude;

use control::*;
use prelude::*;

const DEFAULT_SLAVE_ID: u8 = 255;
const DEFAULT_BAUD_RATE: u32 = 115200;

struct CustomAppState {
    switch_controller: Option<SwitchController>,
}

#[tauri::command(rename_all = "snake_case")]
async fn get_usb_serial_port_list() -> CustomAppResult<Vec<USBSerialPortInfo>> {
    SwitchController::get_usb_serial_port_list().map_to_message()
}

#[tauri::command(rename_all = "snake_case")]
async fn connect_switch(
    serial_port_name: String,
    state: State<'_, Mutex<CustomAppState>>,
) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if custom_app_state.switch_controller.is_some() {
        return Err("请先关闭已连接开关".to_string());
    }

    let switch_controller =
        SwitchController::new(serial_port_name, DEFAULT_BAUD_RATE, DEFAULT_SLAVE_ID)
            .map_to_message()?;

    custom_app_state.switch_controller = Some(switch_controller);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn disconnect_switch(state: State<'_, Mutex<CustomAppState>>) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller.disconnect().await.map_to_message()?;
    } else {
        return Err("未连接开关".to_string());
    }

    custom_app_state.switch_controller = None;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn get_switch_state(state: State<'_, Mutex<CustomAppState>>) -> CustomAppResult<ReadSwitchState> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller.get_switch_state().await.map_to_message()
    } else {
        Err("未连接设备".to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn open_switch(state: State<'_, Mutex<CustomAppState>>) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller.open_switch().await.map_to_message()
    } else {
        Err("未连接设备".to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn close_switch(state: State<'_, Mutex<CustomAppState>>) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller.close_switch().await.map_to_message()
    } else {
        Err("未连接设备".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(CustomAppState {
            switch_controller: None,
        }))
        .invoke_handler(tauri::generate_handler![
            get_usb_serial_port_list,
            connect_switch,
            disconnect_switch,
            get_switch_state,
            open_switch,
            close_switch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

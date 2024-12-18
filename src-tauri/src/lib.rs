use crate::prelude::CustomAppResult;
use tauri::async_runtime::Mutex;
use tauri::{Manager, State};

mod control;
mod prelude;

use control::*;
use prelude::*;

struct CustomAppState {
    switch_controller: Option<SwitchController>,
}

#[tauri::command(rename_all = "snake_case")]
async fn get_usb_serial_port_list() -> CustomAppResult<Vec<USBSerialPortInfo>> {
    SwitchController::get_usb_serial_port_list().map_to_message()
}

#[tauri::command(rename_all = "snake_case")]
async fn connect_switch(
    modbus_config: ModbusConfig,
    state: State<'_, Mutex<CustomAppState>>,
) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller.disconnect().await.map_to_message()?;
    }

    let switch_controller = SwitchController::new(modbus_config).map_to_message()?;

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
async fn get_switch_state(
    state: State<'_, Mutex<CustomAppState>>,
) -> CustomAppResult<ReadSwitchState> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller.get_switch_state().await.map_to_message()
    } else {
        Err("未连接设备".to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn operate_switch(
    state: State<'_, Mutex<CustomAppState>>,
    operation_state: WriteSwitchState,
) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller
            .operate_switch(operation_state)
            .await
            .map_to_message()
    } else {
        Err("未连接设备".to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn get_app_config() -> CustomAppResult<ModbusConfig> {
    SwitchController::get_app_config().await.map_to_message()
}

#[tauri::command(rename_all = "snake_case")]
async fn set_app_config(modbus_config: ModbusConfig) -> CustomAppResult<()> {
    SwitchController::set_app_config(modbus_config)
        .await
        .map_to_message()
}

#[tauri::command(rename_all = "snake_case")]
async fn set_baud_rate(
    state: State<'_, Mutex<CustomAppState>>,
    baud_rate: u32,
) -> CustomAppResult<()> {
    let mut custom_app_state = state.lock().await;

    if let Some(ref mut switch_controller) = custom_app_state.switch_controller {
        switch_controller
            .set_baud_rate(baud_rate)
            .await
            .map_to_message()
    } else {
        Err("未连接设备".to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn custom_init(modbus_config: ModbusConfig) -> CustomAppResult<u8> {
    SwitchController::custom_init(modbus_config)
        .await
        .map_to_message()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_shadow(true)?;
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(CustomAppState {
            switch_controller: None,
        }))
        .invoke_handler(tauri::generate_handler![
            get_usb_serial_port_list,
            connect_switch,
            disconnect_switch,
            get_switch_state,
            operate_switch,
            get_app_config,
            set_app_config,
            set_baud_rate,
            custom_init
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

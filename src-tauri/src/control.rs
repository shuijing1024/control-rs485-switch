use crate::prelude::AnyHowResult;
use anyhow::Context;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;
use tokio_modbus::client::Context as modbusContext;
use tokio_modbus::prelude::{rtu, Client, Reader, Writer};
use tokio_modbus::Result as ModbusResult;
use tokio_modbus::Slave;
use tokio_serial::SerialPortType::UsbPort;
use tokio_serial::{SerialPortInfo, SerialStream};

const DEFAULT_SERIAL_TIMEOUT_MILLIS: u64 = 500;
const DEFAULT_MODBUS_TIMEOUT_MILLIS: u64 = 1000;
const DEFAULT_SWITCH_READ_REGISTER_ADDRESS: u16 = 0x0026;
const DEFAULT_SWITCH_WRITE_REGISTER_ADDRESS: u16 = 0x0101;

#[derive(Serialize)]
pub struct USBSerialPortInfo {
    value: String,
    label: String,
}

pub struct SwitchController {
    modbus_context: modbusContext,
}

#[derive(Serialize, TryFromPrimitive)]
#[repr(u8)]
pub enum ReadSwitchState {
    Close = 0, // 合闸
    Open = 1,  // 分闸
    Lock = 2,
}

#[derive(IntoPrimitive)]
#[repr(u16)]
enum WriteSwitchState {
    Open = 0x01, // 分闸
    Close = 0x02, // 合闸
                 // Lock = 0x03,
                 // Unlock = 0x04,
}

impl SwitchController {
    pub async fn open_switch(&mut self) -> AnyHowResult<()> {
        Self::modbus_action_with_timeout(
            async {
                self.modbus_context
                    .write_single_register(
                        DEFAULT_SWITCH_WRITE_REGISTER_ADDRESS,
                        WriteSwitchState::Open.into(),
                    )
                    .await
            },
            "无法打开开关",
        )
        .await
    }

    pub async fn close_switch(&mut self) -> AnyHowResult<()> {
        Self::modbus_action_with_timeout(
            async {
                self.modbus_context
                    .write_single_register(
                        DEFAULT_SWITCH_WRITE_REGISTER_ADDRESS,
                        WriteSwitchState::Close.into(),
                    )
                    .await
            },
            "无法关闭开关",
        )
        .await
    }

    pub async fn get_switch_state(&mut self) -> AnyHowResult<ReadSwitchState> {
        let register_data_list = Self::modbus_action_with_timeout(
            async {
                self.modbus_context
                    .read_holding_registers(DEFAULT_SWITCH_READ_REGISTER_ADDRESS, 2)
                    .await
            },
            "无法读取开关状态",
        )
        .await?;

        if register_data_list.len() != 2 {
            return Err(anyhow::anyhow!("开关状态数据获取失败"));
        }

        let register_data = (register_data_list[1] & 0x00ff) as u8;

        ReadSwitchState::try_from(register_data).context("无法解析开关状态")
    }

    pub async fn disconnect(&mut self) -> AnyHowResult<()> {
        self.modbus_context
            .disconnect()
            .await
            .context("无法断开连接")
    }
}

impl SwitchController {
    pub fn new(port_name: String, baud_rate: u32, slave_id: u8) -> AnyHowResult<Self> {
        let serial_builder = tokio_serial::new(&port_name, baud_rate).timeout(
            std::time::Duration::from_millis(DEFAULT_SERIAL_TIMEOUT_MILLIS),
        );

        let serial_port =
            SerialStream::open(&serial_builder).context(format!("无法打开串口: {}", port_name))?;

        let slave = Slave(slave_id);

        let modbus_context = rtu::attach_slave(serial_port, slave);

        Ok(Self { modbus_context })
    }

    pub fn get_usb_serial_port_list() -> AnyHowResult<Vec<USBSerialPortInfo>> {
        let serial_list = tokio_serial::available_ports().context("无法获得串口列表")?;

        let mut usb_serial_port_list = Vec::new();

        for port in serial_list {
            if let SerialPortInfo {
                port_name,
                port_type: UsbPort(usb_port_info),
            } = port
            {
                let usb_serial_label = if let Some(product_label) = usb_port_info.product {
                    if product_label.contains(&port_name) {
                        product_label
                    } else {
                        format!("({}) - {}", &port_name, product_label)
                    }
                } else {
                    port_name.clone()
                };

                let info = USBSerialPortInfo {
                    value: port_name,
                    label: usb_serial_label,
                };

                usb_serial_port_list.push(info);
            }
        }

        Ok(usb_serial_port_list)
    }
}

impl SwitchController {
    async fn modbus_action_with_timeout<F, T>(
        action: F,
        description_message: &str,
    ) -> AnyHowResult<T>
    where
        F: std::future::Future<Output = ModbusResult<T>>,
    {
        tokio::select! {
             result = action => {
                if let Ok(Ok(need_message))=result{
                    Ok(need_message)
                }else{
                    Err(anyhow::anyhow!("{}",description_message))
                }
            },
            _ = tokio::time::sleep(std::time::Duration::from_millis(DEFAULT_MODBUS_TIMEOUT_MILLIS))=>Err(anyhow::anyhow!("modbus操作超时")),
        }
    }
}

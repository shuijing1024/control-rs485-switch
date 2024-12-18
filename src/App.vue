<script>
import {invoke} from "@tauri-apps/api/core";
import {message} from '@tauri-apps/plugin-dialog';
import {Close, FullScreen, Minus} from "@element-plus/icons-vue";
import {Window} from '@tauri-apps/api/window';

const appWindow = new Window('main');

export default {
  name: "App",
  components: {FullScreen, Close, Minus},
  data() {
    return {
      loadingState: false,

      selectSerialPort: "",
      serialUSBPortList: [],

      slaveId: 1,
      baudRate: 4800,
      timeout: 6_000,

      resetBaudRate: 4800,

      switch_connect_state: false,
      switch_operate_state: "Open",
    }
  },
  methods: {
    handleWindowsAction(action) {
      switch (action) {
        case "minimize":
          appWindow.minimize();
          break;
        case "maximize":
          appWindow.toggleMaximize();
          break;
        case "close":
          appWindow.close();
          break;
        default:
          break;
      }
    },
    async handleRustCommand(command, errorHandle) {
      const changeLoadingStateTimer = setTimeout(() => {
        this.loadingState = true;
      }, 300);

      try {
        await command();
      } catch (e) {
        if (errorHandle !== undefined) {
          let result = errorHandle();
          if (result !== true) {
            await message(e.toString(), {title: "错误", kind: "error"});
          }
        } else {
          await message(e.toString(), {title: "错误", kind: "error"});
        }
      } finally {
        clearTimeout(changeLoadingStateTimer);
        if (this.loadingState) {
          this.loadingState = false;
        }
      }
    },
    async getSerialUSBPorts() {
      await this.handleRustCommand(async () => {
        this.serialUSBPortList = await invoke("get_usb_serial_port_list");
      })
    },
    async toggleConnectButton() {
      if (!this.switch_connect_state) {
        await this.handleRustCommand(async () => {
          await invoke("connect_switch", {
            modbus_config: {
              port_name: this.selectSerialPort,
              baud_rate: this.baudRate,
              slave_id: this.slaveId,
              timeout: this.timeout,
            }
          });

          this.switch_operate_state = await invoke("get_switch_state");

          this.switch_connect_state = true;

          invoke("set_app_config", {
            modbus_config: {
              port_name: this.selectSerialPort,
              baud_rate: this.baudRate,
              slave_id: this.slaveId,
              timeout: this.timeout,
            }
          }).catch(() => {
          });
        }, () => {
          invoke("disconnect_switch")
              .catch(() => {
              });
        })
      } else {
        await this.handleRustCommand(async () => {
          await invoke("disconnect_switch");
          this.switch_connect_state = false;

          this.serialUSBPortList = await invoke("get_usb_serial_port_list");
        })
      }
    },
    async toggleOperateButton() {
      await this.handleRustCommand(async () => {
        let next_operate_state = "Open";
        switch (this.switch_operate_state) {
          case "Open":
            next_operate_state = "Close";
            break;
          case "Close":
            next_operate_state = "Open"
            break;
          case "Lock":
            next_operate_state = "Unlock";
            break;
          case "Unlock":
            next_operate_state = "Lock"
            break;
          default:
            break;
        }
        await invoke("operate_switch", {operation_state: next_operate_state});
        this.switch_operate_state = next_operate_state
      }, () => {
        invoke("get_usb_serial_port_list")
            .then((result) => {
              this.serialUSBPortList = result;
              let findIndex = this.serialUSBPortList.findIndex((item) => item.value === this.selectSerialPort);
              if (findIndex === -1) {
                this.toggleConnectButton().catch(() => {
                });
              }
            })
            .catch(() => {
            });
      })
    },
    async changeBaudRate() {
      await this.handleRustCommand(async () => {
        await invoke("set_baud_rate", {
          baud_rate: this.resetBaudRate,
        });
      }, () => {
        this.baudRate = this.resetBaudRate;
        this.toggleConnectButton().catch(() => {
        });

        return true;
      });
    }
  },
  computed: {
    toggle_connect_button_enable_flag() {
      if (!this.switch_connect_state) {
        return this.selectSerialPort !== "";
      } else {
        return true;
      }
    },

    toggle_operate_button_display_name() {
      switch (this.switch_operate_state) {
        case "Open":
          return "分闸";
        case "Close":
          return "合闸";
        case "Lock":
          return "解锁";
        case "Unlock":
          return "锁定";
        default:
          return "未知状态";
      }
    }
  },
  mounted() {
    setTimeout(() => {
      invoke("get_usb_serial_port_list")
          .then(portList => {
            this.serialUSBPortList = portList;

            invoke("get_app_config").then(result => {
              let findIndex = portList.findIndex((item) => item.value === result.port_name);
              if (findIndex !== -1) {
                this.selectSerialPort = result.port_name;
              }

              this.baudRate = result.baud_rate;
              this.slaveId = result.slave_id;
              this.timeout = result.timeout;
            })
          })
          .catch(() => {
          })
    }, 1);

  },
}
</script>

<template>
  <div class="custom-container">
    <div data-tauri-drag-region class="titlebar">
      <div class="titlebar-button" id="titlebar-minimize" @click="handleWindowsAction('minimize')">
        <el-icon>
          <Minus/>
        </el-icon>
      </div>
      <div class="titlebar-button" id="titlebar-maximize" @click="handleWindowsAction('maximize')">
        <el-icon>
          <FullScreen/>
        </el-icon>
      </div>
      <div class="titlebar-button" id="titlebar-close" @click="handleWindowsAction('close')">
        <el-icon>
          <Close/>
        </el-icon>
      </div>
    </div>

    <div class="column-center"
         v-loading="loadingState"
         element-loading-text="运行中..."
         element-loading-background="rgba(122, 122, 122, 0.8)"
    >
      <el-container>
        <el-main>
          <el-row>
            <el-col :span="14">
              <el-select
                  v-model="selectSerialPort"
                  placeholder="尚未选择串口"
                  no-data-text="未找到USB串口"
                  size="large"
                  :disabled="switch_connect_state"
              >
                <el-option
                    v-for="item in serialUSBPortList"
                    :key="item.value"
                    :label="item.label"
                    :value="item.value"
                />
              </el-select>
            </el-col>
            <el-col :span="6" :offset="4">
              <el-button type="primary"
                         @click="getSerialUSBPorts"
                         :disabled="switch_connect_state"
                         size="large">刷新串口
              </el-button>
            </el-col>
          </el-row>
          <el-row>
            <el-col :span="4">
              <el-input-number v-model="baudRate" :min="4800" :max="115200" size="large" :step="200"
                               :disabled="switch_connect_state" controls-position="right"/>
            </el-col>
            <el-col :span="4" :offset="1">
              <el-input-number v-model="slaveId" :min="1" :max="255" size="large" :disabled="switch_connect_state"
                               controls-position="right"/>
            </el-col>
            <el-col :span="4" :offset="1">
              <el-input-number v-model="timeout" :min="1000" :max="100_000" size="large" :step="100"
                               :disabled="switch_connect_state" controls-position="right"/>
            </el-col>
            <el-col :span="6" :offset="4">
              <el-button type="primary"
                         @click="toggleConnectButton"
                         :disabled="!toggle_connect_button_enable_flag"
                         size="large">
                {{ switch_connect_state ? "断开" : "连接" }}
              </el-button>
            </el-col>
          </el-row>
          <el-row>
            <el-col :span="4">
              <el-input-number v-model="resetBaudRate" :min="4800" :max="115200" size="large" :step="200"
                               :disabled="!switch_connect_state"
                               controls-position="right"/>
            </el-col>
            <el-col :span="6" :offset="14">
              <el-button type="primary"
                         @click="changeBaudRate"
                         :disabled="!switch_connect_state"
                         size="large">
                重设波特率
              </el-button>
            </el-col>
          </el-row>
          <el-row justify="center">
            <el-col :span="8">
              <el-button type="primary" @click="toggleOperateButton" size="large" :disabled="!switch_connect_state">
                {{ toggle_operate_button_display_name }}
              </el-button>
            </el-col>
          </el-row>
        </el-main>
      </el-container>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 30px;
  user-select: none;
  display: flex;
  justify-content: flex-end;
  top: 0;
  left: 0;
  right: 0;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
  user-select: none;
  -webkit-user-select: none;
}

.titlebar-button:hover {
  background: #5bbec3;
}

.el-row {
  margin-bottom: 30px;
}

.el-row:last-child {
  margin-bottom: 0;
}

.el-col {
  border-radius: 4px;
}

.el-button {
  width: 100%;
}

.el-input-number {
  width: 100%;
}

.custom-container {
  display: flex;
  flex-direction: column;
  height: auto;
  min-height: 100%;
}

.column-center {
  display: flex;
  flex-grow: 1;
  justify-content: center;
  align-items: center;
  position: relative;
}
</style>


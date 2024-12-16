<script>
import {invoke} from "@tauri-apps/api/core";
import {message} from '@tauri-apps/plugin-dialog';

export default {
  name: "App",
  data() {
    return {
      loadingState: false,

      selectSerialPort: "",
      serialUSBPortList: [],

      slaveId: 1,
      baudRate: 115200,

      switch_connect_state: false,
      switch_operate_state: "Open",
    }
  },
  methods: {
    async handleRustCommand(command, errorHandle) {
      const changeLoadingStateTimer = setTimeout(() => {
        this.loadingState = true;
      }, 300);

      try {
        await command();
      } catch (e) {
        if (errorHandle !== undefined) {
          errorHandle();
        }

        await message(e.toString(), {title: "错误", kind: "error"});
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
            serial_port_name: this.selectSerialPort,
            baud_rate: this.baudRate,
            slave_id: this.slaveId
          });

          this.switch_operate_state = await invoke("get_switch_state");

          this.switch_connect_state = true;
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
                this.selectSerialPort = "";
                this.switch_connect_state = false;
              }
            })
            .catch(() => {
            });
      })
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
          })
          .catch(() => {
          })
    }, 1);

  },
}
</script>

<template>
  <div class="column-center"
       v-loading="loadingState"
       element-loading-text="运行中..."
       element-loading-background="rgba(122, 122, 122, 0.8)"
  >
    <el-container>
      <el-main>
        <el-row :gutter="20" justify="space-between">
          <el-col :span="16">
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
          <el-col :span="6">
            <el-button type="primary"
                       @click="getSerialUSBPorts"
                       :disabled="switch_connect_state"
                       size="large">刷新串口
            </el-button>
          </el-col>
        </el-row>
        <el-row :gutter="20" justify="space-between">
          <el-col :span="8">
            <el-input-number v-model="baudRate" :min="4800" :max="115200" size="large" :step="100"
                             :disabled="switch_connect_state" controls-position="right"/>
          </el-col>
          <el-col :span="8">
            <el-input-number v-model="slaveId" :min="1" :max="255" size="large" :disabled="switch_connect_state"
                             controls-position="right"/>
          </el-col>
          <el-col :span="6">
            <el-button type="primary"
                       @click="toggleConnectButton"
                       :disabled="!toggle_connect_button_enable_flag"
                       size="large">
              {{ switch_connect_state ? "断开" : "连接" }}
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
</template>

<style scoped>
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

.column-center {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}
</style>


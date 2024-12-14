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

      switch_connect_state: false,
      switch_open_state: true,  // 为true对应着为Open，即为分闸。为false为Close，即为合闸。
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
          await invoke("connect_switch", {serial_port_name: this.selectSerialPort});
          this.switch_connect_state = true;

          await invoke("open_switch");  // 分闸
          this.switch_open_state = true;

          // const switch_state = await invoke("get_switch_state");
          // switch (switch_state) {
          //   case "Open":
          //     // 为open，分闸状态
          //     this.switch_open_state = true;
          //     break;
          //   case "Close":
          //     // 为close，合闸状态
          //     this.switch_open_state = false;
          //     break;
          //   default:
          //     // 为锁定状态
          //     await message("闸门已被锁定！", {title: "警告", kind: "warning"});
          // }
        })
      } else {
        await this.handleRustCommand(async () => {
          await invoke("disconnect_switch");
          this.switch_connect_state = false;

          this.serialUSBPortList = await invoke("get_usb_serial_port_list");
        })
      }
    },
    async toggleOpenButton() {
      await this.handleRustCommand(async () => {
        if (this.switch_open_state) {
          // 分闸状态
          await invoke("close_switch"); // 合闸
          this.switch_open_state = false;
        } else {
          // 合闸状态
          await invoke("open_switch");  // 分闸
          this.switch_open_state = true;
        }
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
          <el-col :span="4">
            <el-button type="primary"
                       @click="getSerialUSBPorts"
                       :disabled="switch_connect_state"
                       size="large">刷新串口
            </el-button>
          </el-col>
          <el-col :span="4">
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
            <el-button type="primary" @click="toggleOpenButton" size="large" :disabled="!switch_connect_state">
              {{ switch_open_state ? "合闸" : "分闸" }}
              <!--              状态为true，说明为Open，为分闸，那么点击就是合闸。-->
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


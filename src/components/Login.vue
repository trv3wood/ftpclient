<script setup lang="ts">
import { computed, Ref, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ErrorKind } from "../type";

const host = ref("")
const username = ref("");
const passwd = ref("")
const port = ref(21)
const msg: Ref<string | null> = ref(null)
const status = ref(false);
const isLoading = ref(false)
const msgClass = computed(() => {
  return status.value ? 'alert-success' : 'alert-danger'
})
async function doLogin(host: string, name: string, passwd: string, port: number) {
  isLoading.value = true
  msg.value = null

  // 示例验证
  if (!host || !name || !passwd) {
    throw new Error('请填写完整信息')
  }

  try {
    const result = await invoke('login', {
      host,
      name,
      passwd,
      port
    });

    console.log('登录成功:', result);
    // 处理成功逻辑
    status.value = true;
    msg.value = '登录成功！';
  } catch (e) {
    console.error('调用命令时发生意外错误:', e as ErrorKind);
    msg.value = (e as ErrorKind).message
    status.value = false;
  } finally {
    isLoading.value = false
  }

}

async function login() {
  isLoading.value = true
  msg.value = null

  await doLogin(host.value, username.value, passwd.value, port.value)
}
</script>

<template>
  <div class="container mt-5">
    <div class="row justify-content-center">
      <div class="col-md-10">
        <h1 class="text-center mb-4">FTP客户端</h1>

        <form @submit.prevent="login" class="card p-4 shadow-sm form-control">
          <div class="mb-3">
            <label for="host" class="form-label">主机地址</label>
            <input id="host" v-model="host" type="text" class="form-control" placeholder="例如: ftp.example.com" required>
          </div>

          <div class="mb-3">
            <label for="username" class="form-label">用户名</label>
            <input id="username" v-model="username" type="text" class="form-control" placeholder="输入用户名" required>
          </div>

          <div class="mb-3">
            <label for="passwd" class="form-label">密码</label>
            <input id="passwd" v-model="passwd" type="password" class="form-control" placeholder="输入密码" required>
          </div>

          <div class="mb-3">
            <label for="port" class="form-label">端口</label>
            <input id="port" v-model="port" type="number" class="form-control" placeholder="默认21" min="1" max="65535">
          </div>

          <button type="submit" class="btn btn-primary w-100">
            <span v-if="!isLoading">连接</span>
            <span v-else class="spinner-border spinner-border-sm" role="status"></span>
          </button>

          <div v-if="msg" class="alert mt-3" :class="msgClass">{{ msg }}</div>
        </form>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  border-radius: 0.5rem;
  background-color: aliceblue;
}

.spinner-border {
  vertical-align: middle;
}
</style>

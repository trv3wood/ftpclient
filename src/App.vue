<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const host = ref("")
const username = ref("");
const passwd = ref("")
const port = ref("")
const msg = ref("")
const isLoading = ref(false)
const msgClass = computed(() => ({
  'alert-success': msg.value.includes('成功'),
  'alert-danger': msg.value.includes('失败') || msg.value.includes('错误')
}))

async function login() {
  try {
    isLoading.value = true
    msg.value = ''

    // 示例验证
    if (!host.value || !username.value || !passwd.value) {
      throw new Error('请填写完整信息')
    }

    msg.value = await invoke('login', { host: host.value, name: username.value, passwd: passwd.value, port: port.value })
  } catch (error) {
    msg.value = error instanceof Error ? error.message : '连接失败'
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <main class="container mt-5">
    <div class="row justify-content-center">
      <div class="col-md-6">
        <h1 class="text-center mb-4">FTP客户端</h1>

        <form @submit.prevent="login" class="card p-4 shadow-sm">
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
  </main>
</template>

<style scoped>
.card {
  border-radius: 0.5rem;
}

.spinner-border {
  vertical-align: middle;
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: white;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

  button:active {
    background-color: #0f0f0f69;
  }
}
</style>
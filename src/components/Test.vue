<template>
  <div class="container">
    <h1>测试组件</h1>
    <button @click="testLogin">测试登录</button>
    <button @click="testPwd">当前目录</button>
  </div>
</template>
<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core';

const testLogin = async () => {
  await invoke('login', { host: '127.0.0.1', name: 'ftp', passwd: '1', port: 2121 })
}
const testPwd = async () => {
  const res = await invoke('pwd')
  console.log(res)
}
</script>
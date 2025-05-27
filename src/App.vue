<script setup lang="ts">
import { ref } from 'vue';
import Login from './components/Login.vue';
import Home from './components/Home.vue';
import { invoke } from '@tauri-apps/api/core';
import Test from './components/Test.vue';
const activeButton = ref('connect');
const tabs = [
  { name: 'connect', label: '连接' },
  { name: 'file', label: '文件' },
  { name: 'test', label: '测试' }
];
async function quit() {
  // 退出逻辑
  console.log('退出登录');
  await invoke('logout');
}
</script>
<template>
  <main class="container">
    <div class="sidebar d-flex flex-column">
      <button v-for="tab in tabs" :key="tab.name" :class="{ active: activeButton === tab.name }"
        @click="activeButton = tab.name">
        {{ tab.label }}
      </button>
      <button @click="quit" class="btn quitbtn">
        断开连接
      </button>
    </div>
    <div class="view">
      <div v-if="activeButton === 'connect'">
        <Login />
      </div>
      <div v-else-if="activeButton === 'file'">
        <Home />
      </div>
      <div v-else-if="activeButton === 'test'">
        <Test />
      </div>
    </div>
  </main>
</template>
<style scoped>
.container {
  margin: 0 auto;
  padding: 20px;
}

.sidebar {
  position: fixed;
  top: 0;
  left: 0;
  width: 20%;
  height: 100%;
}

.sidebar button {
  padding: 1em;
  border: none;
  margin: 1vw 1vw;
  border-radius: 15px;
  background-color: transparent;
}

.sidebar .quitbtn {
  position: absolute;
  bottom: 0;
  left: 0;
  width: calc(100% - 20px);
  background-color: red;
  color: white;
}

.sidebar button.active {
  background-color: lightblue;
  color: white;
}

.sidebar button:hover {
  background-color: gray;
  color: black;
}

.view {
  position: relative;
  margin-top: 0;
  margin-left: 20%;
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
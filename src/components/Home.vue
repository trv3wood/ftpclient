<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'
import { ErrorKind } from '../type'
import { open } from '@tauri-apps/plugin-dialog';

const currentPath = ref('.')
const list = ref<string[]>([])
const errMessage = ref<ErrorKind | null>(null)
onMounted(async () => {
    errMessage.value = null
    loadDirectory(currentPath.value).catch((error) => {
        const err = error as ErrorKind
        errMessage.value = err
    })
})
// 加载目录内容
async function loadDirectory(path: string) {
    // 清除错误信息
    errMessage.value = null
    // 更新数据
    currentPath.value = await invoke<string>('pwd')
    // 调用后端 API
    list.value = await invoke<string[]>('nls', { path: path })
}

async function download(fileName: string) {
    try {
        const save_path = await invoke('download', { file: fileName })
        alert(`文件 ${fileName} 已保存到: ${save_path}`)
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}

async function changeDir(path: string) {
    // 如果是目录，进入该目录
    try {
        await invoke('cd', { path: path })
        await loadDirectory('.')
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}
async function uploadfile() {
    try {
        const filePath = await open({
            multiple: false,
            directory: false,
            filters: [{
                name: 'All Files',
                extensions: ['*']
            }]
        })
        invoke('upload', { file: filePath })
        await loadDirectory('.')
    } catch (error) {
        console.error(error)
        const err = error as ErrorKind
        errMessage.value = err
    }
}

// 返回上级目录
async function goBack() {
    try {
        await invoke('cd', { path: '..' })
        await loadDirectory('.')
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}
</script>

<template>
    <div class="container">
        <div v-if="errMessage" class="alert alert-danger">
            {{ errMessage.kind }}: {{ errMessage.message }}
        </div>
        <div class="navigation-bar">
            <button @click="goBack" class="btn btn-sm btn-secondary">
                ← 返回
            </button>
            <span class="current-path">当前路径: {{ currentPath }}</span>
            <button @click="uploadfile" class="btn btn-sm btn-success float-end">
                上传文件
            </button>
        </div>

        <ul class="list-group">
            <li v-for="(item, index) in list" :key="index"
                class="list-group-item d-flex justify-content-between align-items-center">
                <label>{{ item }}</label>
                <button v-if="item.endsWith('/')" class="btn btn-sm btn-secondary" @click="changeDir(item)">进入</button>
                <button v-else class="btn btn-sm btn-primary" @click="download(item)">下载</button>
            </li>
        </ul>
    </div>
</template>

<style scoped>
.container {
    margin: auto;
    padding: 40px;
}

.navigation-bar {
    margin: 20px 0;
    padding: 10px;
    background: #f8f9fa;
    border-radius: 4px;
}

.current-path {
    margin-left: 15px;
    font-weight: bold;
}

.list-group-item {
    cursor: pointer;
    transition: background-color 0.2s;
}

.list-group-item.is-directory:hover {
    background-color: #e9ecef;
}

.badge {
    font-size: 0.75em;
}
</style>

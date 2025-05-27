<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'
import { ErrorKind } from '../type'

const currentPath = ref('.')
const list = ref<string[]>([])
const historyStack = ref<string[]>([]) // 用于记录导航历史
onMounted(async () => {
    await loadDirectory(currentPath.value)
})
// 加载目录内容
async function loadDirectory(path: string) {
    try {
        // 更新数据
        invoke<string>('pwd').then((res: string) => {
            currentPath.value = res
        }).catch((error) => {
            const err = error as ErrorKind
            console.error('获取当前路径失败:', err.kind, err.message)
        })

        // 调用后端 API
        invoke<string[]>('nls', { path: path }).then((files) => {
            list.value = files
        }).catch((error) => {
            let err = error as ErrorKind
            alert(err.kind + err.message)
        })

        // 记录历史
        historyStack.value.push(path)
    } catch (error) {
        console.error('加载目录失败:', error)
    }
}


// 返回上级目录
async function goBack() {
    if (historyStack.value.length > 1) {
        historyStack.value.pop()
        const prevPath = historyStack.value[historyStack.value.length - 1]
        await invoke('cd', { path: prevPath })
        await loadDirectory(prevPath)
    }
}
</script>

<template>
    <div class="container">
        <div class="navigation-bar">
            <button @click="goBack" class="btn btn-sm btn-secondary" :disabled="historyStack.length <= 1">
                ← 返回
            </button>
            <span class="current-path">当前路径: {{ currentPath }}</span>
        </div>

        <ul class="list-group">
            <li v-for="(item, index) in list" :key="index"
                class="list-group-item d-flex justify-content-between align-items-center">
                {{ item }}
            </li>
        </ul>
    </div>
</template>

<style scoped>
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

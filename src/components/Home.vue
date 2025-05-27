<template>
    <div class="container">
        <label>当前路径 {{ currentPath }}</label>
        <ul class="list-group">
            <li v-for="(item, index) in list" :key="index" class="list-group-item">
                {{ item }}
            </li>
        </ul>
    </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { onMounted, ref } from 'vue';

const currentPath = ref<string>('');
const list = ref<string[]>([]);
onMounted(async () => {
        invoke<string>('pwd').then((res: string) => {
            currentPath.value = res.trim();
            console.log('获取当前路径成功:', currentPath.value);
        })
        .catch((err) => {
            console.error('获取当前路径失败:', err);
            return undefined;
        });
    invoke<string>('ls').then((res: string) => {
        res = res.trim();
        if (res) {
            list.value = res.split('\n').map(item => item.trim()).filter(item => item);
        } else {
            list.value = [];
        }
        console.log('获取文件列表成功:', list.value);
    }).catch((err) => {
        console.error('获取文件列表失败:', err);
    });
})
</script>
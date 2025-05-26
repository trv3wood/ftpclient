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
const list = ref<string[]>(["src", "assets", "components", "views"]);
onMounted(async () => {
    currentPath.value = await invoke('pwd')
})
</script>
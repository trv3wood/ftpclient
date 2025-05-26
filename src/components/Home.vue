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
import { ref } from 'vue';

const currentPath = ref<string>('');
const list = ref<string[]>(["src", "assets", "components", "views"]);
</script>
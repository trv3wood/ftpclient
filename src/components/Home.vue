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

async function rm(params: { path: string }) {
    try {
        await invoke('rm', { path: params.path })
        await loadDirectory('.')
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}

async function mkdir(params: { path: string }) {
    try {
        await invoke('mkdir', { path: params.path })
        await loadDirectory('.')
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}

async function rename(params: { oldPath: string, newPath: string }) {
    try {
        await invoke('rename', { oldPath: params.oldPath, newPath: params.newPath })
        await loadDirectory('.')
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}

async function rmdir(params: { path: string }) {
    try {
        await invoke('rmdir', { path: params.path })
        await loadDirectory('.')
    } catch (error) {
        const err = error as ErrorKind
        errMessage.value = err
    }
}

const showCreateDirModal = ref(false);
const newDirName = ref('');
const showRenameModalFlag = ref(false);
const newName = ref('');
const selectedItem = ref('');
const showDeleteConfirm = ref(false);
const isDirectory = ref(false);

// 创建目录
async function createDir() {
    try {
        await mkdir({ path: newDirName.value });
        showCreateDirModal.value = false;
        newDirName.value = '';
    } catch (error) {
        console.error(error);
    }
}

// 显示重命名模态框
function showRenameModal(item: string) {
    selectedItem.value = item;
    isDirectory.value = item.endsWith('/');
    showRenameModalFlag.value = true;
    newName.value = '';
}

// 执行重命名
async function renameItem() {
    try {
        await rename({
            oldPath: selectedItem.value,
            newPath: newName.value
        });
        showRenameModalFlag.value = false;
    } catch (error) {
        console.error(error);
    }
}

// 确认删除文件
function confirmDeleteFile(item: string) {
    selectedItem.value = item;
    isDirectory.value = false;
    showDeleteConfirm.value = true;
}

// 确认删除目录
function confirmDeleteDir(item: string) {
    selectedItem.value = item;
    isDirectory.value = true;
    showDeleteConfirm.value = true;
}

// 执行删除
async function deleteItem() {
    try {
        if (isDirectory.value) {
            await rmdir({ path: selectedItem.value });
        } else {
            await rm({ path: selectedItem.value });
        }
        showDeleteConfirm.value = false;
    } catch (error) {
        console.error(error);
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
            <div class="float-end">
                <button @click="showCreateDirModal = true" class="btn btn-sm btn-success me-2">
                    新建目录
                </button>
                <button @click="uploadfile" class="btn btn-sm btn-success">
                    上传文件
                </button>
            </div>
        </div>

        <ul class="list-group">
            <li v-for="(item, index) in list" :key="index"
                class="list-group-item d-flex justify-content-between align-items-center">
                <label>{{ item }}</label>
                <div>
                    <button v-if="item.endsWith('/')" class="btn btn-sm btn-secondary me-2" @click="changeDir(item)">
                        进入
                    </button>
                    <button class="btn btn-sm btn-warning me-2" @click="showRenameModal(item)">
                        重命名
                    </button>
                    <button v-if="item.endsWith('/')" class="btn btn-sm btn-danger me-2"
                        @click="confirmDeleteDir(item)">
                        删除目录
                    </button>
                    <button v-else class="btn btn-sm btn-danger me-2" @click="confirmDeleteFile(item)">
                        删除文件
                    </button>
                    <button v-if="!item.endsWith('/')" class="btn btn-sm btn-primary" @click="download(item)">
                        下载
                    </button>
                </div>
            </li>
        </ul>

        <!-- 创建目录模态框 -->
        <div v-if="showCreateDirModal" class="modal" style="display: block; background: rgba(0,0,0,0.5)">
            <div class="modal-dialog">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">新建目录</h5>
                        <button type="button" class="btn-close" @click="showCreateDirModal = false"></button>
                    </div>
                    <div class="modal-body">
                        <input v-model="newDirName" type="text" class="form-control" placeholder="输入目录名">
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" @click="showCreateDirModal = false">取消</button>
                        <button type="button" class="btn btn-primary" @click="createDir">创建</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 重命名模态框 -->
        <div v-if="showRenameModalFlag" class="modal" style="display: block; background: rgba(0,0,0,0.5)">
            <div class="modal-dialog">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">重命名</h5>
                        <button type="button" class="btn-close" @click="showRenameModalFlag = false"></button>
                    </div>
                    <div class="modal-body">
                        <input v-model="newName" type="text" class="form-control" :placeholder="selectedItem">
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" @click="showRenameModalFlag = false">取消</button>
                        <button type="button" class="btn btn-primary" @click="renameItem">确认</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 确认删除模态框 -->
        <div v-if="showDeleteConfirm" class="modal" style="display: block; background: rgba(0,0,0,0.5)">
            <div class="modal-dialog">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">确认删除</h5>
                        <button type="button" class="btn-close" @click="showDeleteConfirm = false"></button>
                    </div>
                    <div class="modal-body">
                        <p>确定要删除 "{{ selectedItem }}" 吗？</p>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" @click="showDeleteConfirm = false">取消</button>
                        <button type="button" class="btn btn-danger" @click="deleteItem">确认删除</button>
                    </div>
                </div>
            </div>
        </div>
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

.modal {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 1050;
    overflow-x: hidden;
    overflow-y: auto;
    outline: 0;
}

.modal-dialog {
    position: relative;
    width: auto;
    margin: 0.5rem;
    pointer-events: none;
}

.modal-content {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100%;
    pointer-events: auto;
    background-color: #fff;
    background-clip: padding-box;
    border: 1px solid rgba(0, 0, 0, .2);
    border-radius: 0.3rem;
    outline: 0;
}

.modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid #dee2e6;
}

.modal-body {
    position: relative;
    flex: 1 1 auto;
    padding: 1rem;
}

.modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 1rem;
    border-top: 1px solid #dee2e6;
}

.btn-close {
    padding: 0.5rem;
    margin: -0.5rem -0.5rem -0.5rem auto;
    background-color: transparent;
    border: 0;
    border-radius: 0.25rem;
    opacity: .5;
}
</style>

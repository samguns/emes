<script setup lang="ts">
import { ref } from 'vue';
import ChatInterface from './ChatInterface.vue';
import ToolExplorer from './ToolExplorer.vue';
import ResourceBrowser from './ResourceBrowser.vue';
import playMusic from './playMusic.vue';

// 控制playMusic组件的显示
const showPlayMusic = ref(false);

function handleClickPlayMusic() {
  if (!showPlayMusic.value) {
    showPlayMusic.value = true;
  } else {
    showPlayMusic.value = false;
  }
}

// State
const activeTab = ref<'chat' | 'tools' | 'resources'>('chat');

// Methods
function setActiveTab(tab: 'chat' | 'tools' | 'resources') {
  activeTab.value = tab;
}
</script>

<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <h1>宸宝的智能助手</h1>
    </div>

    <div class="dashboard-content">

      <div class="card">
        <!-- <h2>Quick Actions</h2> -->
        <div class="action-buttons">
          <button class="action-button" @click="handleClickPlayMusic">播放音乐</button>
          <button class="action-button">Action 2</button>
          <button class="action-button">Action 3</button>
        </div>
      </div>
    </div>

    <!-- 播放音乐组件 -->
    <div v-if="showPlayMusic" class="music-container">
      <playMusic />
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.dashboard-header {
  padding: 1rem;
  background-color: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
}

.dashboard-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.dashboard-content {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.music-container {
  margin-top: 1rem;
  width: 100%;
}

.card {
  background-color: var(--color-background-soft);
  border-radius: 8px;
  padding: 1rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.card h2 {
  margin-top: 0;
  font-size: 1.2rem;
  margin-bottom: 0.75rem;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
}

.activity-icon {
  font-size: 1.5rem;
}

.activity-details h3 {
  margin: 0;
  font-size: 1rem;
}

.activity-details p {
  margin: 0.25rem 0 0;
  font-size: 0.9rem;
  color: var(--color-text-light);
}

.action-buttons {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 0.5rem;
}

.action-button {
  background-color: var(--color-primary);
  /* color: white; */
  border: none;
  border-radius: 4px;
  padding: 0.5rem;
  font-size: 0.9rem;
  cursor: pointer;
}

/* Mobile-specific styles */
@media (max-width: 768px) {
  .dashboard-header h1 {
    font-size: 1.25rem;
  }
  
  .card h2 {
    font-size: 1.1rem;
  }
  
  .action-buttons {
    grid-template-columns: 1fr 1fr;
  }
}
</style>
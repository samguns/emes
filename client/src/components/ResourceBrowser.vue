<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { mcpService } from '../services/MCPService';

// State
const resources = ref<any[]>([]);
const selectedResource = ref<string | null>(null);
const resourceContent = ref<any>(null);
const isLoading = ref(false);
const errorMessage = ref('');

// Fetch resources when component mounts or MCP connection changes
watch(() => mcpService.isConnected, async (isConnected) => {
  if (isConnected) {
    await fetchResources();
  } else {
    resources.value = [];
    selectedResource.value = null;
    resourceContent.value = null;
  }
}, { immediate: true });

// Fetch available resources
async function fetchResources() {
  if (!mcpService.isConnected) return;
  
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    await mcpService.refreshCapabilities();
    resources.value = mcpService.availableResources;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

// Fetch resource content
async function fetchResourceContent(uri: string) {
  if (!mcpService.isConnected) return;
  
  selectedResource.value = uri;
  resourceContent.value = null;
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    resourceContent.value = await mcpService.getResource(uri);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

// Format resource content for display
function formatContent(content: any): string {
  if (!content) return 'No content';
  
  if (typeof content === 'string') {
    return content;
  }
  
  if (content.type === 'text') {
    return content.text || 'Empty text';
  }
  
  if (content.type === 'image') {
    return `[Image: ${content.alt || 'No description'}]`;
  }
  
  return JSON.stringify(content, null, 2);
}
</script>

<template>
  <div class="resource-browser">
    <div class="browser-header">
      <h2>Resource Browser</h2>
      <button @click="fetchResources" :disabled="isLoading || !mcpService.isConnected" class="refresh-button">
        {{ isLoading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>
    
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>
    
    <div class="browser-content">
      <div class="resources-list">
        <div v-if="resources.length === 0 && !isLoading" class="empty-state">
          No resources available
        </div>
        
        <div 
          v-for="resource in resources" 
          :key="resource.uri"
          :class="['resource-item', { active: selectedResource === resource.uri }]"
          @click="fetchResourceContent(resource.uri)"
        >
          <div class="resource-name">{{ resource.name || 'Unnamed resource' }}</div>
          <div class="resource-uri">{{ resource.uri }}</div>
        </div>
      </div>
      
      <div class="resource-viewer">
        <div v-if="!selectedResource" class="empty-state">
          Select a resource to view its content
        </div>
        
        <div v-else-if="isLoading" class="loading-state">
          Loading resource content...
        </div>
        
        <div v-else class="content-display">
          <pre>{{ formatContent(resourceContent) }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.resource-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.browser-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background-color: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
}

.browser-header h2 {
  margin: 0;
}

.refresh-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-primary);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.refresh-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  padding: 0.5rem 1rem;
  background-color: #ffebee;
  color: #c62828;
  border-bottom: 1px solid #ef9a9a;
}

.browser-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.resources-list {
  width: 30%;
  min-width: 250px;
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
}

.resource-item {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
}

.resource-item:hover {
  background-color: var(--color-background-mute);
}

.resource-item.active {
  background-color: var(--color-background-mute);
  border-left: 3px solid var(--color-primary);
}

.resource-name {
  font-weight: bold;
  margin-bottom: 0.25rem;
}

.resource-uri {
  font-size: 0.8rem;
  color: var(--color-text-light);
  word-break: break-all;
}

.resource-viewer {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
}

.empty-state, .loading-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: var(--color-text-light);
  font-style: italic;
}

.content-display {
  white-space: pre-wrap;
  word-break: break-word;
}

pre {
  margin: 0;
  white-space: pre-wrap;
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  .browser-content {
    flex-direction: column;
  }
  
  .resources-list {
    width: 100%;
    max-height: 40%;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
  }
}
</style>
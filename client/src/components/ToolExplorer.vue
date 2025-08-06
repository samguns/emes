<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { mcpService } from '../services/MCPService';

// State
const tools = ref<any[]>([]);
const selectedToolName = ref<string | null>(null);
const toolArguments = ref('{}');
const executionResult = ref<any>(null);
const isLoading = ref(false);
const errorMessage = ref('');

// Computed properties
const selectedTool = computed(() => {
  if (!selectedToolName.value) return null;
  return tools.value.find(tool => tool.name === selectedToolName.value);
});

// Watch for MCP connection changes
watch(() => mcpService.isConnected, async (isConnected) => {
  if (isConnected) {
    await fetchTools();
  } else {
    tools.value = [];
    selectedToolName.value = null;
    executionResult.value = null;
  }
}, { immediate: true });

// Fetch available tools
async function fetchTools() {
  if (!mcpService.isConnected) return;
  
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    await mcpService.refreshCapabilities();
    tools.value = mcpService.availableTools;
    
    if (tools.value.length > 0 && !selectedToolName.value) {
      selectedToolName.value = tools.value[0].name;
      updateArgumentsTemplate();
    }
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

// Update the arguments template when a tool is selected
function updateArgumentsTemplate() {
  if (!selectedTool.value) return;
  
  const tool = selectedTool.value;
  if (!tool.parameters) {
    toolArguments.value = '{}';
    return;
  }
  
  // Create a template with default values
  const template: Record<string, any> = {};
  
  if (tool.parameters.properties) {
    Object.entries(tool.parameters.properties).forEach(([key, prop]: [string, any]) => {
      if (prop.default !== undefined) {
        template[key] = prop.default;
      } else if (prop.type === 'string') {
        template[key] = '';
      } else if (prop.type === 'number' || prop.type === 'integer') {
        template[key] = 0;
      } else if (prop.type === 'boolean') {
        template[key] = false;
      } else if (prop.type === 'array') {
        template[key] = [];
      } else if (prop.type === 'object') {
        template[key] = {};
      }
    });
  }
  
  toolArguments.value = JSON.stringify(template, null, 2);
}

// Execute the selected tool
async function executeTool() {
  if (!mcpService.isConnected || !selectedToolName.value) return;
  
  isLoading.value = true;
  errorMessage.value = '';
  executionResult.value = null;
  
  try {
    let args = {};
    try {
      args = JSON.parse(toolArguments.value);
    } catch (e) {
      errorMessage.value = `Invalid JSON arguments: ${e instanceof Error ? e.message : String(e)}`;
      isLoading.value = false;
      return;
    }
    
    executionResult.value = await mcpService.callTool(selectedToolName.value, args);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

// Format tool result for display
function formatResult(result: any): string {
  if (!result) return 'No result';
  
  if (typeof result === 'string') {
    return result;
  }
  
  if (result.content && Array.isArray(result.content)) {
    return result.content.map((item: any) => {
      if (item.type === 'text') {
        return item.text || '';
      } else if (item.type === 'image') {
        return `[Image: ${item.alt || 'No description'}]`;
      } else {
        return JSON.stringify(item, null, 2);
      }
    }).join('\n');
  }
  
  return JSON.stringify(result, null, 2);
}
</script>

<template>
  <div class="tool-explorer">
    <div class="explorer-header">
      <h2>Tool Explorer</h2>
      <button @click="fetchTools" :disabled="isLoading || !mcpService.isConnected" class="refresh-button">
        {{ isLoading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>
    
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>
    
    <div class="explorer-content">
      <div class="tools-panel">
        <div class="tool-selector">
          <label for="tool-select">Select Tool:</label>
          <select 
            id="tool-select" 
            v-model="selectedToolName"
            @change="updateArgumentsTemplate"
            :disabled="isLoading || tools.length === 0"
          >
            <option 
              v-for="tool in tools" 
              :key="tool.name" 
              :value="tool.name"
            >
              {{ tool.name }}
            </option>
          </select>
        </div>
        
        <div v-if="selectedTool" class="tool-description">
          <p>{{ selectedTool.description || 'No description available' }}</p>
        </div>
        
        <div class="tool-arguments">
          <label for="tool-args">Arguments (JSON):</label>
          <textarea 
            id="tool-args" 
            v-model="toolArguments" 
            rows="8"
            :disabled="isLoading || !selectedToolName"
          ></textarea>
        </div>
        
        <button 
          @click="executeTool" 
          :disabled="isLoading || !selectedToolName"
          class="execute-button"
        >
          {{ isLoading ? 'Executing...' : 'Execute Tool' }}
        </button>
      </div>
      
      <div class="result-panel">
        <h3>Result</h3>
        <div v-if="isLoading" class="loading-state">
          Executing tool...
        </div>
        <div v-else-if="executionResult" class="result-content">
          <pre>{{ formatResult(executionResult) }}</pre>
        </div>
        <div v-else class="empty-state">
          Execute a tool to see results
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tool-explorer {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.explorer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background-color: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
}

.explorer-header h2 {
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

.explorer-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.tools-panel {
  width: 40%;
  min-width: 300px;
  padding: 1rem;
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.tool-selector {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tool-selector label {
  font-weight: bold;
}

.tool-selector select {
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--color-background);
  color: var(--color-text);
}

.tool-description {
  font-size: 0.9rem;
  color: var(--color-text-light);
}

.tool-arguments {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tool-arguments label {
  font-weight: bold;
}

.tool-arguments textarea {
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--color-background);
  color: var(--color-text);
  font-family: monospace;
  resize: vertical;
}

.execute-button {
  padding: 0.75rem;
  background-color: var(--color-primary);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.execute-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.result-panel {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.result-panel h3 {
  margin-top: 0;
  margin-bottom: 1rem;
}

.loading-state, .empty-state {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  color: var(--color-text-light);
  font-style: italic;
}

.result-content {
  background-color: var(--color-background-soft);
  padding: 1rem;
  border-radius: 4px;
  overflow-x: auto;
}

.result-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  .explorer-content {
    flex-direction: column;
  }
  
  .tools-panel {
    width: 100%;
    max-height: 50%;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
  }
}
</style>
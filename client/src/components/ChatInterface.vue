<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { mcpService } from '../services/MCPService';

// State
const messages = ref<Array<{role: string, content: string, timestamp: Date}>>([]);
const inputMessage = ref('');
const isConnected = ref(false);
const isConnecting = ref(false);
const connectionUrl = ref('http://localhost:8642/mcp');
const connectionType = ref('streamableHttp'); // Options: websocket, http, streamableHttp
const availableTools = ref<any[]>([]);
const selectedTool = ref('');
const toolArguments = ref('{}');
const connectionError = ref('');

// Connect to MCP server
async function connectToServer() {
  if (isConnected.value || isConnecting.value) return;
  
  connectionError.value = '';
  isConnecting.value = true;
  
  try {
    let connectionConfig: any = {};
    
    if (connectionType.value === 'websocket') {
      connectionConfig = {
        type: "websocket",
        url: connectionUrl.value.startsWith('ws') ? connectionUrl.value : connectionUrl.value.replace('http', 'ws'),
      };
    } else if (connectionType.value === 'http') {
      connectionConfig = {
        type: "http",
        url: connectionUrl.value.startsWith('http') ? connectionUrl.value : connectionUrl.value.replace('ws', 'http'),
      };
    } else if (connectionType.value === 'streamableHttp') {
      connectionConfig = {
        type: "streamableHttp",
        url: connectionUrl.value.startsWith('http') ? connectionUrl.value : connectionUrl.value.replace('ws', 'http'),
      };
    }

    await mcpService.connect(connectionConfig);
    
    // Set up event listeners
    mcpService.onLoggingMessage((message: any) => {
      messages.value.push({
        role: 'system',
        content: `[LOG ${message.level}] ${message.data}`,
        timestamp: new Date()
      });
    });

    // Get available tools
    await fetchTools();
    
    isConnected.value = true;
    messages.value.push({
      role: 'system',
      content: 'Connected to MCP server successfully',
      timestamp: new Date()
    });
  } catch (error) {
    connectionError.value = error instanceof Error ? error.message : String(error);
    messages.value.push({
      role: 'system',
      content: `Connection error: ${connectionError.value}`,
      timestamp: new Date()
    });
  } finally {
    isConnecting.value = false;
  }
}

// Disconnect from MCP server
async function disconnect() {
  if (!isConnected.value) return;
  
  try {
    await mcpService.disconnect();
    isConnected.value = false;
    messages.value.push({
      role: 'system',
      content: 'Disconnected from MCP server',
      timestamp: new Date()
    });
  } catch (error) {
    messages.value.push({
      role: 'system',
      content: `Disconnect error: ${error instanceof Error ? error.message : String(error)}`,
      timestamp: new Date()
    });
  }
}

// Fetch available tools from the server
async function fetchTools() {
  if (!isConnected.value) return;
  
  try {
    await mcpService.refreshCapabilities();
    availableTools.value = mcpService.availableTools;
    if (availableTools.value.length > 0) {
      selectedTool.value = availableTools.value[0].name;
    }
  } catch (error) {
    messages.value.push({
      role: 'system',
      content: `Error fetching tools: ${error instanceof Error ? error.message : String(error)}`,
      timestamp: new Date()
    });
  }
}

// Execute a tool
async function executeTool() {
  if (!isConnected.value || !selectedTool.value) return;
  
  try {
    let args = {};
    try {
      args = JSON.parse(toolArguments.value);
    } catch (e) {
      messages.value.push({
        role: 'system',
        content: `Invalid JSON arguments: ${e instanceof Error ? e.message : String(e)}`,
        timestamp: new Date()
      });
      return;
    }
    
    messages.value.push({
      role: 'user',
      content: `Executing tool: ${selectedTool.value} with args: ${toolArguments.value}`,
      timestamp: new Date()
    });
    
    const result = await mcpService.callTool(selectedTool.value, args);
    
    // Process the result
    let resultContent = '';
    if (result && result.content) {
      resultContent = result.content.map((item: any) => {
        if (item.type === 'text') {
          return item.text;
        } else if (item.type === 'image') {
          return `[Image: ${item.alt || 'No description'}]`;
        } else {
          return JSON.stringify(item);
        }
      }).join('\n');
    } else {
      resultContent = JSON.stringify(result);
    }
    
    messages.value.push({
      role: 'assistant',
      content: resultContent,
      timestamp: new Date()
    });
  } catch (error) {
    messages.value.push({
      role: 'system',
      content: `Error executing tool: ${error instanceof Error ? error.message : String(error)}`,
      timestamp: new Date()
    });
  }
}

// Send a message
async function sendMessage() {
  if (!inputMessage.value.trim()) return;
  
  const userMessage = inputMessage.value;
  messages.value.push({
    role: 'user',
    content: userMessage,
    timestamp: new Date()
  });
  
  inputMessage.value = '';
  
  // If not connected, try to connect first
  if (!isConnected.value) {
    await connectToServer();
    if (!isConnected.value) return; // Connection failed
  }
  
  // For now, we'll just echo the message back
  // In a real implementation, you would process the message through MCP
  messages.value.push({
    role: 'assistant',
    content: `You said: ${userMessage}`,
    timestamp: new Date()
  });
}

// Format timestamp
function formatTime(date: Date): string {
  return date.toLocaleTimeString();
}

// Cleanup on component unmount
onUnmounted(async () => {
  if (isConnected.value) {
    await disconnect();
  }
});
</script>

<template>
  <div class="chat-interface">
    <div class="connection-panel">
      <div class="connection-form">
        <div class="form-group">
          <label for="connection-url">MCP Server URL:</label>
          <input 
            id="connection-url" 
            v-model="connectionUrl" 
            type="text" 
            :disabled="isConnected || isConnecting"
          />
        </div>
        
        <div class="form-group">
          <label for="connection-type">Connection Type:</label>
          <select 
            id="connection-type" 
            v-model="connectionType"
            :disabled="isConnected || isConnecting"
          >
            <option value="websocket">WebSocket</option>
            <option value="http">HTTP</option>
            <option value="streamableHttp">Streamable HTTP</option>
          </select>
        </div>
        
        <div class="connection-actions">
          <button 
            v-if="!isConnected" 
            @click="connectToServer" 
            :disabled="isConnecting"
            class="connect-button"
          >
            {{ isConnecting ? 'Connecting...' : 'Connect' }}
          </button>
          <button 
            v-else 
            @click="disconnect"
            class="disconnect-button"
          >
            Disconnect
          </button>
        </div>
        
        <div v-if="connectionError" class="connection-error">
          {{ connectionError }}
        </div>
      </div>
    </div>
    
    <div class="tools-panel" v-if="isConnected">
      <h3>Available Tools</h3>
      <div class="tools-form">
        <div class="form-group">
          <label for="tool-select">Select Tool:</label>
          <select id="tool-select" v-model="selectedTool">
            <option 
              v-for="tool in availableTools" 
              :key="tool.name" 
              :value="tool.name"
            >
              {{ tool.name }}
            </option>
          </select>
        </div>
        
        <div class="form-group">
          <label for="tool-args">Arguments (JSON):</label>
          <textarea id="tool-args" v-model="toolArguments" rows="3"></textarea>
        </div>
        
        <button @click="executeTool" class="execute-button">Execute Tool</button>
      </div>
    </div>
    
    <div class="chat-container">
      <div class="messages-container">
        <div 
          v-for="(message, index) in messages" 
          :key="index"
          :class="['message', message.role]"
        >
          <div class="message-header">
            <span class="message-role">{{ message.role }}</span>
            <span class="message-time">{{ formatTime(message.timestamp) }}</span>
          </div>
          <div class="message-content">
            {{ message.content }}
          </div>
        </div>
      </div>
      
      <div class="input-container">
        <input 
          v-model="inputMessage" 
          @keyup.enter="sendMessage" 
          type="text" 
          placeholder="Type a message..."
        />
        <button @click="sendMessage">Send</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat-interface {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background-color: var(--color-background);
}

.connection-panel {
  padding: 1rem;
  background-color: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
}

.connection-form {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
}

.form-group {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 200px;
}

.form-group label {
  margin-bottom: 0.25rem;
  font-size: 0.9rem;
}

.form-group input,
.form-group select,
.form-group textarea {
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--color-background);
  color: var(--color-text);
}

.connection-actions {
  display: flex;
  align-items: flex-end;
}

.connect-button,
.disconnect-button,
.execute-button {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.connect-button {
  background-color: var(--color-primary);
  color: white;
}

.disconnect-button {
  background-color: #e74c3c;
  color: white;
}

.execute-button {
  background-color: var(--color-primary);
  color: white;
}

.connection-error {
  color: #e74c3c;
  margin-top: 0.5rem;
  width: 100%;
}

.tools-panel {
  padding: 1rem;
  background-color: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
}

.tools-panel h3 {
  margin-top: 0;
  margin-bottom: 0.5rem;
}

.tools-form {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.messages-container {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.message {
  padding: 0.5rem;
  border-radius: 8px;
  max-width: 80%;
}

.message.user {
  align-self: flex-end;
  background-color: var(--color-primary);
  color: white;
}

.message.assistant {
  align-self: flex-start;
  background-color: var(--color-background-soft);
}

.message.system {
  align-self: center;
  background-color: var(--color-background-mute);
  font-style: italic;
  font-size: 0.9rem;
}

.message-header {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  margin-bottom: 0.25rem;
}

.message-role {
  font-weight: bold;
}

.message-time {
  opacity: 0.7;
}

.input-container {
  display: flex;
  padding: 1rem;
  border-top: 1px solid var(--color-border);
}

.input-container input {
  flex: 1;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 4px 0 0 4px;
  background-color: var(--color-background);
  color: var(--color-text);
}

.input-container button {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 0 4px 4px 0;
  background-color: var(--color-primary);
  color: white;
  cursor: pointer;
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  .connection-form,
  .tools-form {
    flex-direction: column;
  }
  
  .form-group {
    width: 100%;
  }
  
  .message {
    max-width: 90%;
  }
}
</style>
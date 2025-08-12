import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { McpServerInfo, McpTool, McpResource } from '../types';

export const useMcpStore = defineStore('mcp', () => {
    const isConnected = ref(false);
    const isConnecting = ref(false);
    const serverInfo = ref<McpServerInfo | null>(null);
    const tools = ref<McpTool[]>([]);
    const resources = ref<McpResource[]>([]);
    const error = ref<string | null>(null);

    const isReady = computed(() => {
        return isConnected.value && !isConnecting.value;
    });
    const hasTools = computed(() => {
        return tools.value.length > 0;
    });
    const hasResources = computed(() => {
        return resources.value.length > 0;
    });

    function setConnected(connected: boolean) {
        isConnected.value = connected
        if (!connected) {
          serverInfo.value = null
          tools.value = []
          resources.value = []
        }
      }
    
      function setConnecting(connecting: boolean) {
        isConnecting.value = connecting
      }
    
      function setServerInfo(info: McpServerInfo) {
        serverInfo.value = info
      }
    
      function setTools(newTools: McpTool[]) {
        tools.value = newTools
      }
    
      function setResources(newResources: McpResource[]) {
        resources.value = newResources
      }
    
      function setError(errorMessage: string | null) {
        error.value = errorMessage
      }

    return {
        isConnected,
        isConnecting,
        serverInfo,
        tools,
        resources,
        error,

        isReady,
        hasTools,
        hasResources,

        setConnected,
        setConnecting,
        setServerInfo,
        setTools,
        setResources,
        setError,
    }
});
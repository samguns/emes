export interface MCPMessage {
    jsonrpc: '2.0';
    id?: string | number;
    method?: string;
    params?: any;
    result?: any;
    error?: {
        code: number;
        message: string;
        data?: any;
    };
}

export interface MCPConnectionConfig {
    type: 'websocket' | 'http' | 'streamableHttp';
    url: string;
    headers?: Record<string, string>;
}

export class BrowserMCPClient {
    private ws: WebSocket | null = null;
    private messageId = 0;
    private pendingRequests = new Map<string | number, { resolve: Function; reject: Function }>();
    private eventListeners = new Map<string, Function[]>();
    private httpStreamController: AbortController | null = null;
    private httpStreamReader: ReadableStreamDefaultReader<Uint8Array> | null = null;
    private currentUrl: string = '';
    private isConnected = false;
    private sessionId: string | null = null;
    private protocolVersion: string = '2025-06-18';

    constructor(
        private name: string,
        private version: string
    ) { }

    async connect(config: MCPConnectionConfig): Promise<void> {
        if (config.type === 'websocket') {
            return this.connectWebSocket(config.url);
        } else if (config.type === 'http') {
            return this.connectHTTP(config.url, config.headers);
        } else if (config.type === 'streamableHttp') {
            return this.connectStreamableHTTP(config.url, config.headers);
        }
        throw new Error('Unsupported connection type');
    }

    private async connectWebSocket(url: string): Promise<void> {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(url);

            this.ws.onopen = () => {
                this.initialize().then(resolve).catch(reject);
            };

            this.ws.onerror = (error) => {
                reject(new Error(`WebSocket connection failed: ${error}`));
            };

            this.ws.onmessage = (event) => {
                this.handleMessage(JSON.parse(event.data));
            };

            this.ws.onclose = () => {
                this.emit('disconnected');
            };
        });
    }

    private async connectHTTP(url: string, headers?: Record<string, string>): Promise<void> {
        // For HTTP, we'll simulate the connection
        // In a real implementation, you might use Server-Sent Events or polling
        console.log('HTTP connection not fully implemented - using WebSocket fallback');
        return this.connectWebSocket(url.replace('http', 'ws'));
    }

    private async connectStreamableHTTP(url: string, headers?: Record<string, string>): Promise<void> {
        try {
            this.currentUrl = url;
            console.log('Connecting to streamable HTTP server:', url);

            // Send initialization request
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json, text/event-stream',
                    'Cache-Control': 'no-cache',
                    'Connection': 'keep-alive',
                    ...headers
                },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    id: 1,
                    method: 'initialize',
                    params: {
                        protocolVersion: this.protocolVersion,
                        capabilities: {
                            tools: {},
                            resources: {},
                            prompts: {}
                        },
                        clientInfo: {
                            name: this.name,
                            version: this.version
                        }
                    }
                })
            });

            if (!response.ok) {
                throw new Error(`HTTP connection failed: ${response.status} ${response.statusText}`);
            }

            // Read the Mcp-Session-Id header from the response
            this.sessionId = response.headers.get('Mcp-Session-Id');

            this.setupStreamFromResponse(response);
            this.isConnected = true;

            // Send notifications/initialized notification
            await this.sendNotificationsInitialized();

        } catch (error) {
            console.error('Streamable HTTP connection failed:', error);
            throw error;
        }
    }

    private setupStreamFromResponse(response: Response): void {
        const contentType = response.headers.get('content-type');
        console.log('Response content-type:', contentType);

        if (contentType?.includes('application/json')) {
            // Handle immediate JSON response
            response.json().then(data => {
                this.handleMessage(data);
            }).catch(error => {
                console.error('Failed to parse JSON response:', error);
            });
        } else if (response.body) {
            // Set up stream reading
            this.httpStreamController = new AbortController();
            this.httpStreamReader = response.body.getReader();

            // Start reading the stream
            this.readStream();
        }

        console.log('Streamable HTTP connection established');
    }

    private async readStream(): Promise<void> {
        if (!this.httpStreamReader) return;

        try {
            while (true) {
                const { done, value } = await this.httpStreamReader.read();

                if (done) {
                    console.log('Stream ended');
                    this.emit('disconnected');
                    break;
                }

                // Convert the chunk to text
                const chunk = new TextDecoder().decode(value);
                console.log('Received chunk:', chunk);

                const lines = chunk.split('\n').filter(line => line.trim());

                for (const line of lines) {
                    // Strip the data: prefix
                    const data = line.replace('data: ', '');
                    try {
                        const message = JSON.parse(data);
                        console.log('Parsed message:', message);
                        this.handleMessage(message);
                    } catch (error) {
                        console.warn('Failed to parse message:', line, error);
                    }
                }
            }
        } catch (error) {
            console.error('Error reading stream:', error);
            this.emit('disconnected');
        }
    }

    private async initialize(): Promise<void> {
        const response = await this.sendRequest('initialize', {
            protocolVersion: this.protocolVersion,
            capabilities: {
                tools: {},
                resources: {},
                prompts: {}
            },
            clientInfo: {
                name: this.name,
                version: this.version
            }
        });

        if (response.error) {
            throw new Error(`Initialization failed: ${response.error.message}`);
        }
    }

    private async sendRequest(method: string, params?: any): Promise<MCPMessage> {
        const id = ++this.messageId;
        const message: MCPMessage = {
            jsonrpc: '2.0',
            id,
            method,
            params
        };

        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });

            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(JSON.stringify(message));
            } else if (this.httpStreamReader) {
                // For streamable HTTP, send the request via a separate HTTP call
                this.sendStreamableHTTPRequest(method, params).then(resolve).catch(reject);
            } else {
                reject(new Error('No connection available'));
            }
        });
    }

    private async sendStreamableHTTPRequest(method: string, params?: any): Promise<MCPMessage> {
        const id = ++this.messageId;
        const message: MCPMessage = {
            jsonrpc: '2.0',
            id,
            method,
            params
        };

        // console.log('Sending streamable HTTP request:', message);

        // Prepare headers with MCP-specific headers
        const headers: Record<string, string> = {
            'Content-Type': 'application/json',
            'Accept': 'application/json, text/event-stream',
            'Cache-Control': 'no-cache',
            'Connection': 'keep-alive',
            'MCP-Protocol-Version': this.protocolVersion,
        };

        // Include session ID if available
        if (this.sessionId) {
            headers['Mcp-Session-Id'] = this.sessionId;
        }

        try {
            const response = await fetch(this.currentUrl, {
                method: 'POST',
                headers,
                body: JSON.stringify(message)
            });

            if (!response.ok) {
                throw new Error(`HTTP request failed: ${response.status} ${response.statusText}`);
            }

            // Read the response as a stream if possible, otherwise as JSON
            if (response.headers.get('content-type')?.includes('text/event-stream')) {
                // Read the stream and accumulate the result
                const reader = response.body?.getReader();
                let result = '';
                if (reader) {
                    const decoder = new TextDecoder();
                    while (true) {
                        const { done, value } = await reader.read();
                        if (done) break;
                        result += decoder.decode(value, { stream: true });
                    }
                    // Try to parse the last event as JSON
                    // (Assume the last event is the full JSON-RPC response)
                    // Find the last non-empty line
                    const lines = result.split('\n').map(l => l.trim()).filter(Boolean);
                    let lastData = lines.reverse().find(line => line.startsWith('data:'));
                    if (lastData) {
                        lastData = lastData.replace(/^data:\s*/, '');
                        try {
                            return JSON.parse(lastData);
                        } catch (e) {
                            throw new Error('Failed to parse event stream response as JSON');
                        }
                    } else {
                        throw new Error('No data event found in event stream');
                    }
                } else {
                    throw new Error('No response body to read from');
                }
            } else {
                // Fallback to normal JSON
                return await response.json();
            }
        } catch (error) {
            console.error('Streamable HTTP request failed:', error);
            throw error;
        }
    }

    private async sendNotificationsInitialized(): Promise<void> {
        const notification = {
            jsonrpc: '2.0',
            method: 'notifications/initialized',
            params: {}
        };

        console.log('Sending notifications/initialized:', notification);

        // Prepare headers with MCP-specific headers
        const headers: Record<string, string> = {
            'Content-Type': 'application/json',
            'Accept': 'application/json, text/event-stream',
            'Cache-Control': 'no-cache',
            'Connection': 'keep-alive',
            'MCP-Protocol-Version': this.protocolVersion,
        };

        // Include session ID if available
        if (this.sessionId) {
            headers['Mcp-Session-Id'] = this.sessionId;
        }

        try {
            const response = await fetch(this.currentUrl, {
                method: 'POST',
                headers,
                body: JSON.stringify(notification)
            });

            if (!response.ok) {
                console.warn('Failed to send notifications/initialized:', response.status, response.statusText);
            } else {
                console.log('Successfully sent notifications/initialized');
            }
        } catch (error) {
            console.error('Failed to send notifications/initialized:', error);
        }
    }

    private handleMessage(message: MCPMessage): void {
        console.log('Handling message:', message);

        if (message.id && this.pendingRequests.has(message.id)) {
            const { resolve, reject } = this.pendingRequests.get(message.id)!;
            this.pendingRequests.delete(message.id);

            if (message.error) {
                reject(new Error(message.error.message));
            } else {
                resolve(message);
            }
        } else if (message.method === 'notifications/message') {
            this.emit('loggingMessage', message.params);
        }
    }

    async ping(): Promise<boolean> {
        try {
            await this.sendRequest('ping');
            return true;
        } catch {
            return false;
        }
    }

    async getAllTools(): Promise<any[]> {
        try {
            const response = await this.sendRequest('tools/list');
            return response.result?.tools || [];
        } catch (error) {
            console.error('Failed to get tools:', error);
            throw error;
        }
    }

    async callTool(name: string, args: Record<string, any>): Promise<any> {
        try {
            const response = await this.sendRequest('tools/call', {
                name,
                arguments: args
            });
            return response.result;
        } catch (error) {
            console.error(`Failed to call tool ${name}:`, error);
            throw error;
        }
    }

    async getAllResources(): Promise<any[]> {
        try {
            const response = await this.sendRequest('resources/list');
            return response.result?.resources || [];
        } catch (error) {
            console.error('Failed to get resources:', error);
            throw error;
        }
    }

    async getResource(uri: string): Promise<any> {
        try {
            const response = await this.sendRequest('resources/read', { uri });
            return response.result;
        } catch (error) {
            console.error(`Failed to get resource ${uri}:`, error);
            throw error;
        }
    }

    async getAllPrompts(): Promise<any[]> {
        try {
            const response = await this.sendRequest('prompts/list');
            return response.result?.prompts || [];
        } catch (error) {
            console.error('Failed to get prompts:', error);
            throw error;
        }
    }

    async complete(ref: any, argument: any[]): Promise<any> {
        try {
            const response = await this.sendRequest('prompts/complete', {
                ref,
                argument
            });
            return response.result;
        } catch (error) {
            console.error('Failed to complete prompt:', error);
            throw error;
        }
    }

    async setLoggingLevel(level: string): Promise<void> {
        try {
            await this.sendRequest('logging/setLevel', { level });
        } catch (error) {
            console.error('Failed to set logging level:', error);
            throw error;
        }
    }

    on(event: string, callback: Function): void {
        if (!this.eventListeners.has(event)) {
            this.eventListeners.set(event, []);
        }
        this.eventListeners.get(event)!.push(callback);
    }

    private emit(event: string, data?: any): void {
        const listeners = this.eventListeners.get(event);
        if (listeners) {
            listeners.forEach(callback => callback(data));
        }
    }

    async disconnect(): Promise<void> {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }

        if (this.httpStreamController) {
            this.httpStreamController.abort();
            this.httpStreamController = null;
        }

        if (this.httpStreamReader) {
            await this.httpStreamReader.cancel();
            this.httpStreamReader = null;
        }

        this.isConnected = false;
        this.sessionId = null;
        this.pendingRequests.clear();
        this.eventListeners.clear();
    }

    get connected(): boolean {
        return this.isConnected;
    }
} 
import { BrowserMCPClient } from './BrowserMCPClient';
import type { MCPConnectionConfig } from './BrowserMCPClient';

// Re-export the interface for convenience
export type { MCPConnectionConfig } from './BrowserMCPClient';

export class MCPService {
    private client: BrowserMCPClient | null = null;
    private _isConnected: boolean = false;
    private _availableTools: any[] = [];
    private _availableResources: any[] = [];
    private _availablePrompts: any[] = [];

    constructor() { }

    get isConnected(): boolean {
        return this._isConnected;
    }

    get availableTools(): any[] {
        return this._availableTools;
    }

    get availableResources(): any[] {
        return this._availableResources;
    }

    get availablePrompts(): any[] {
        return this._availablePrompts;
    }

    async connect(config: MCPConnectionConfig): Promise<void> {
        if (this._isConnected) {
            throw new Error('Already connected to an MCP server');
        }

        this.client = new BrowserMCPClient(
            "EMES MCP Client",
            "1.0.0"
        );

        await this.client.connect(config);
        this._isConnected = true;

        // Fetch available capabilities
        await this.refreshCapabilities();
    }

    async disconnect(): Promise<void> {
        if (!this._isConnected || !this.client) {
            return;
        }

        await this.client.disconnect();
        this._isConnected = false;
        this.client = null;
        this._availableTools = [];
        this._availableResources = [];
        this._availablePrompts = [];
    }

    async refreshCapabilities(): Promise<void> {
        if (!this._isConnected || !this.client) {
            throw new Error('Not connected to an MCP server');
        }

        try {
            this._availableTools = await this.client.getAllTools();
        } catch (error) {
            console.error('Failed to fetch tools:', error);
        }

        try {
            this._availableResources = await this.client.getAllResources();
        } catch (error) {
            console.error('Failed to fetch resources:', error);
        }

        try {
            this._availablePrompts = await this.client.getAllPrompts();
        } catch (error) {
            console.error('Failed to fetch prompts:', error);
        }
    }

    async callTool(name: string, args: Record<string, any>): Promise<any> {
        if (!this._isConnected || !this.client) {
            throw new Error('Not connected to an MCP server');
        }

        return await this.client.callTool(name, args);
    }

    async getResource(uri: string): Promise<any> {
        if (!this._isConnected || !this.client) {
            throw new Error('Not connected to an MCP server');
        }

        return await this.client.getResource(uri);
    }

    async executePrompt(name: string, args: Record<string, any>): Promise<any> {
        if (!this._isConnected || !this.client) {
            throw new Error('Not connected to an MCP server');
        }

        return await this.client.complete(
            { type: "ref/prompt", name },
            Object.entries(args).map(([name, value]) => ({ name, value }))
        );
    }

    onLoggingMessage(callback: (message: any) => void): void {
        if (!this.client) return;

        this.client.on('loggingMessage', callback);
    }

    async setLoggingLevel(level: string): Promise<void> {
        if (!this._isConnected || !this.client) {
            throw new Error('Not connected to an MCP server');
        }

        await this.client.setLoggingLevel(level);
    }

    async ping(): Promise<boolean> {
        if (!this._isConnected || !this.client) {
            return false;
        }

        try {
            await this.client.ping();
            return true;
        } catch (error) {
            console.error('Ping failed:', error);
            return false;
        }
    }
}

export const mcpService = new MCPService();
export class MockMCPServer {
    private ws: WebSocket | null = null;
    private messageId = 0;

    constructor(private port: number = 8080) { }

    start(): void {
        // In a real implementation, this would start a WebSocket server
        // For now, we'll just log that the server is ready
        console.log(`Mock MCP Server ready on port ${this.port}`);
        console.log('Connect to: ws://localhost:8080/mcp');
    }

    // Mock responses for testing
    static getMockTools() {
        return [
            {
                name: 'echo',
                description: 'Echo back the input message',
                inputSchema: {
                    type: 'object',
                    properties: {
                        message: {
                            type: 'string',
                            description: 'Message to echo back'
                        }
                    },
                    required: ['message']
                }
            },
            {
                name: 'add',
                description: 'Add two numbers together',
                inputSchema: {
                    type: 'object',
                    properties: {
                        a: {
                            type: 'number',
                            description: 'First number'
                        },
                        b: {
                            type: 'number',
                            description: 'Second number'
                        }
                    },
                    required: ['a', 'b']
                }
            },
            {
                name: 'getWeather',
                description: 'Get weather information for a location',
                inputSchema: {
                    type: 'object',
                    properties: {
                        location: {
                            type: 'string',
                            description: 'Location to get weather for'
                        }
                    },
                    required: ['location']
                }
            }
        ];
    }

    static getMockResources() {
        return [
            {
                name: 'Sample Document',
                uri: 'file:///sample.txt',
                description: 'A sample text document'
            },
            {
                name: 'Configuration',
                uri: 'file:///config.json',
                description: 'Application configuration file'
            }
        ];
    }

    static getMockPrompts() {
        return [
            {
                name: 'greeting',
                description: 'Generate a greeting message',
                arguments: [
                    {
                        name: 'name',
                        description: 'Name to greet',
                        schema: { type: 'string' }
                    }
                ]
            }
        ];
    }
} 
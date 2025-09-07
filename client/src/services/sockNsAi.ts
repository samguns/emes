import { io, Socket } from 'socket.io-client';
import { ref, reactive } from 'vue';

// Types for socket events
interface ServerToClientEvents {
  // Define your server-to-client events here
  message: (data: any) => void;
  error: (error: string) => void;
  connected: () => void;
  disconnected: () => void;
  // Add more events as needed
}

interface ClientToServerEvents {
  // Define your client-to-server events here
  message: (data: any) => void;
  // Add more events as needed
}

export interface SocketState {
  connected: boolean;
  connecting: boolean;
  error: string | null;
  lastConnected: Date | null;
  reconnectAttempts: number;
}

class SockNsAiService {
  private socket: Socket<ServerToClientEvents, ClientToServerEvents> | null = null;
  private readonly serverUrl = 'ws://' + window.location.hostname + ':8642'; // Adjust port if needed
  private readonly namespace = '/ai';
  
  // Reactive state
  public state = reactive<SocketState>({
    connected: false,
    connecting: false,
    error: null,
    lastConnected: null,
    reconnectAttempts: 0
  });

  // Event listeners storage
  private eventListeners: Map<string, Function[]> = new Map();

  constructor() {
    // Auto-connect on instantiation (optional)
    if (!this.socket?.connected) {
      this.connect();
    }
  }

  /**
   * Connect to the socket.io server
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.socket?.connected) {
        resolve();
        return;
      }

      this.state.connecting = true;
      this.state.error = null;

      try {
        this.socket = io(`${this.serverUrl}${this.namespace}`, {
          transports: ['websocket', 'polling'],
          timeout: 20000,
          reconnection: true,
          reconnectionDelay: 1000,
          reconnectionDelayMax: 5000,
          reconnectionAttempts: 5
        });

        // Connection event handlers
        this.socket.on('connect', () => {
          // console.log('Connected to AI namespace');
          this.state.connected = true;
          this.state.connecting = false;
          this.state.error = null;
          this.state.lastConnected = new Date();
          this.state.reconnectAttempts = 0;
          this.emit('connected');
          resolve();
        });

        this.socket.on('disconnect', (reason) => {
          console.log('Disconnected from AI namespace:', reason);
          this.state.connected = false;
          this.state.connecting = false;
          this.emit('disconnected', reason);
        });

        this.socket.on('connect_error', (error) => {
          console.error('Connection error:', error);
          this.state.connected = false;
          this.state.connecting = false;
          this.state.error = error.message;
          this.state.reconnectAttempts++;
          this.emit('error', error.message);
          reject(error);
        });

        this.socket.on('reconnect' as any, (attemptNumber: number) => {
          console.log('Reconnected after', attemptNumber, 'attempts');
          this.state.reconnectAttempts = attemptNumber;
          this.emit('reconnected', attemptNumber);
        });

        this.socket.on('reconnect_error' as any, (error: Error) => {
          console.error('Reconnection error:', error);
          this.state.error = error.message;
          this.emit('reconnect_error', error.message);
        });

        this.socket.on('reconnect_failed' as any, () => {
          console.error('Reconnection failed');
          this.state.error = 'Reconnection failed';
          this.emit('reconnect_failed');
        });

        // Handle custom events
        this.socket.on('message', (data) => {
          this.emit('message', data);
        });

        this.socket.on('error', (error) => {
          console.error('Socket error:', error);
          this.state.error = error;
          this.emit('error', error);
        });

      } catch (error) {
        this.state.connecting = false;
        this.state.error = error instanceof Error ? error.message : 'Connection failed';
        reject(error);
      }
    });
  }

  /**
   * Disconnect from the socket.io server
   */
  disconnect(): void {
    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
      this.state.connected = false;
      this.state.connecting = false;
      console.log('Manually disconnected from AI namespace');
    }
  }

  /**
   * Send a message to the server
   */
  sendMessage(event: string, data?: any): void {
    if (!this.socket?.connected) {
      console.warn('Cannot send message: socket not connected');
      return;
    }

    this.socket.emit(event as any, data);
    console.log('Sent message:', event, data);
  }

  /**
   * Add an event listener
   */
  on(event: string, callback: Function): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(callback);
  }

  /**
   * Remove an event listener
   */
  off(event: string, callback?: Function): void {
    if (!this.eventListeners.has(event)) return;

    if (callback) {
      const listeners = this.eventListeners.get(event)!;
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    } else {
      this.eventListeners.delete(event);
    }
  }

  /**
   * Emit an event to all listeners
   */
  private emit(event: string, ...args: any[]): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(...args);
        } catch (error) {
          console.error(`Error in event listener for ${event}:`, error);
        }
      });
    }
  }

  /**
   * Get connection status
   */
  isConnected(): boolean {
    return this.socket?.connected || false;
  }

  /**
   * Get the socket instance (use with caution)
   */
  getSocket(): Socket<ServerToClientEvents, ClientToServerEvents> | null {
    return this.socket;
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.disconnect();
    this.eventListeners.clear();
  }
}

// Create a singleton instance
const sockNsAiService = new SockNsAiService();

// Export the service instance and class
export default sockNsAiService;
export { SockNsAiService };

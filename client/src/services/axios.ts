import axios from 'axios';

const apiBase = import.meta.env.VITE_API_BASE || 'http://localhost:8642';
console.log('apiBase', apiBase);
export const api = axios.create({
  baseURL: apiBase,
  headers: { 'Content-Type': 'application/json' },
});

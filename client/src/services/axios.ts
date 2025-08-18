import axios from 'axios';

const apiBase = import.meta.env.VITE_API_BASE || '';
export const api = axios.create({
  baseURL: apiBase,
  headers: { 'Content-Type': 'application/json' },
});

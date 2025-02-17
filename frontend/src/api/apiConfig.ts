import axios from 'axios';

const baseURL: string = import.meta.env.VITE_BASE_URL ?? "http://localhost:3000/api";

const apiClient = axios.create({
  baseURL: baseURL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

apiClient.interceptors.request.use(config => {
  // Проверяем, нужно ли добавлять PIN
  if (config.headers['use-pin']) {
    const pin = localStorage.getItem('panel_pin');
    if (pin) {
      config.headers['x-password'] = pin;
    }
    delete config.headers['use-pin']; // Удаляем кастомный флаг
  }
  return config;
});

export default apiClient;
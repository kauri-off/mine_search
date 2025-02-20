import axios from 'axios';

const apiClient = axios.create({
  baseURL: '/api/v1',
  // baseURL: 'http://localhost:3000/api/v1', // for dev
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

apiClient.interceptors.request.use(config => {
  // Проверяем, нужно ли добавлять PIN
  if (config.headers['use-auth']) {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers['Authorization'] = token;
    }
    delete config.headers['use-auth']; // Удаляем кастомный флаг
  }
  return config;
});

export default apiClient;
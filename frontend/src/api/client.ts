import axios from 'axios';
import type { 
    AuthBody, AuthReturn, ServerListRequest, ServerResponse, 
    StatsReturn, DataResponse, ServerDataRequest, UpdateServerBody 
} from '../types';

const API_URL = '/api/v1';

export const api = axios.create({
    baseURL: API_URL,
    headers: {
        'Content-Type': 'application/json',
    },
    withCredentials: true,
});

api.interceptors.response.use(
    (response) => response,
    (error) => {
        const isLoginRequest = error.config?.url?.includes('/auth/login');

        if (error.response?.status === 401 && !isLoginRequest) {
            window.location.href = '/login';
        }
        
        return Promise.reject(error);
    }
);

export const authApi = {
    login: async (body: AuthBody) => {
        const { data } = await api.post<AuthReturn>('/auth/login', body);
        return data;
    },
};

export const serverApi = {
    fetchStats: async () => {
        const { data } = await api.post<StatsReturn>('/stats', {});
        return data;
    },
    fetchList: async (body: ServerListRequest) => {
        const { data } = await api.post<ServerResponse[]>('/servers/list', body);
        return data;
    },
    fetchInfo: async (ip: string) => {
        const { data } = await api.post<ServerResponse>('/server/info', { ip });
        return data;
    },
    fetchData: async (body: ServerDataRequest) => {
        const { data } = await api.post<DataResponse[]>('/server/data', body);
        return data;
    },
    update: async (body: UpdateServerBody) => {
        return api.post('/server/update', body);
    }
};
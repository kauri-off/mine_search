import axios from 'axios';
import type { AuthBody } from '../types/AuthBody';
import type { AuthReturn } from '../types/AuthReturn';
import type { StatsResponse } from '../types/StatsResponse';
import type { ServerDataRequest } from '../types/ServerDataRequest';
import type { UpdateRequest } from '../types/UpdateRequest';
import type { ServerListRequest } from '../types/ServerListRequest';
import type { ServerInfoResponse } from '../types/ServerInfoResponse';
import type { ServerDataResponse } from '../types/ServerDataResponse';

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
        const { data } = await api.post<StatsResponse>('/stats', {});
        return data;
    },
    fetchList: async (body: ServerListRequest) => {
        const { data } = await api.post<ServerInfoResponse[]>('/servers/list', body);
        return data;
    },
    fetchInfo: async (ip: string) => {
        const { data } = await api.post<ServerInfoResponse>('/server/info', { ip });
        return data;
    },
    fetchData: async (body: ServerDataRequest) => {
        const { data } = await api.post<ServerDataResponse[]>('/server/data', body);
        return data;
    },
    update: async (body: UpdateRequest) => {
        return api.post('/server/update', body);
    }
};
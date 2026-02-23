import axios from 'axios';
import type { AuthBody } from '../types/AuthBody';
import type { StatsResponse } from '../types/StatsResponse';
import type { ServerDataRequest } from '../types/ServerDataRequest';
import type { ServerListRequest } from '../types/ServerListRequest';
import type { ServerInfoResponse } from '../types/ServerInfoResponse';
import type { ServerDataResponse } from '../types/ServerDataResponse';
import type { AddIpRequest } from '../types/AddIpRequest';
import type { ServerDeleteRequest } from '../types/ServerDeleteRequest';
import type { UpdateServerRequest } from '../types/UpdateServerRequest';
import type { PlayerResponse } from '../types/PlayerResponse';
import type { PlayerListRequest } from '../types/PlayerListRequest';
import type { UpdatePlayerRequest } from '../types/UpdatePlayerRequest';

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
        await api.post('/auth/login', body);
    },
};

export const serverApi = {
    fetchStats: async () => {
        const { data } = await api.post<StatsResponse>('/stats', {});
        return data;
    },
    fetchServerList: async (body: ServerListRequest) => {
        const { data } = await api.post<ServerInfoResponse[]>('/server/list', body);
        return data;
    },
    fetchServerInfo: async (ip: string) => {
        const { data } = await api.post<ServerInfoResponse>('/server/info', { ip });
        return data;
    },
    fetchServerData: async (body: ServerDataRequest) => {
        const { data } = await api.post<ServerDataResponse[]>('/server/data', body);
        return data;
    },
    updateServer: async (body: UpdateServerRequest) => {
        return api.post('/server/update', body);
    },
    addServerIp: async (body: AddIpRequest) => {
        return api.post('/ip/add', body);
    },
    deleteServer: async (body: ServerDeleteRequest) => {
        return api.post('/server/delete', body);
    },
    fetchPlayerList: async (body: PlayerListRequest) => {
        const { data } = await api.post<PlayerResponse[]>('/player/list', body);
        return data;
    },
    updatePlayer: async (body: UpdatePlayerRequest) => {
        return api.post('/player/update', body);
    },
};
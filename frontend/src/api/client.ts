import axios from 'axios';
import type {
  AuthBody,
  StatsResponse,
  ServerSnapshotsRequest,
  ServerListRequest,
  ServerInfoResponse,
  ServerSnapshotsResponse,
  AddAddrRequest,
  ServerDeleteRequest,
  UpdateServerRequest,
  PlayerResponse,
  PlayerListRequest,
  UpdatePlayerRequest,
  PingServerRequest
} from '@/types';

const API_URL = '/api/v1';

export const api = axios.create({
  baseURL: API_URL,
  headers: { 'Content-Type': 'application/json' },
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
  },
);

export const authApi = {
  login: (body: AuthBody) => api.post('/auth/login', body),
};

export const serverApi = {
  fetchStats: async (): Promise<StatsResponse> => {
    const { data } = await api.post<StatsResponse>('/stats', {});
    return data;
  },

  fetchServerList: async (body: ServerListRequest): Promise<ServerInfoResponse[]> => {
    const { data } = await api.post<ServerInfoResponse[]>('/server/list', body);
    return data;
  },

  fetchServerInfo: async (ip: string): Promise<ServerInfoResponse> => {
    const { data } = await api.post<ServerInfoResponse>('/server/info', { ip });
    return data;
  },

  fetchServerSnapshots: async (body: ServerSnapshotsRequest): Promise<ServerSnapshotsResponse[]> => {
    const { data } = await api.post<ServerSnapshotsResponse[]>('/server/snapshots', body);
    return data;
  },

  updateServer: (body: UpdateServerRequest) => api.post('/server/update', body),

  addTarget: (body: AddAddrRequest) => api.post('/target/add', body),

  deleteServer: (body: ServerDeleteRequest) => api.post('/server/delete', body),

  fetchPlayerList: async (body: PlayerListRequest): Promise<PlayerResponse[]> => {
    const { data } = await api.post<PlayerResponse[]>('/player/list', body);
    return data;
  },

  updatePlayer: (body: UpdatePlayerRequest) => api.post('/player/update', body),

  pingServer: (body: PingServerRequest) => api.post('/server/ping', body),  
};
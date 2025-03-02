import { FiltersList } from "../components/Filters/Filters.types";
import apiClient from "./apiConfig";

export const fetchServerInfo = (ip: string) => {
    return apiClient.post("/server/info", {ip});
}

export const fetchServerList = (limit: number, offset_ip: string | null, filters: FiltersList) => {
    return apiClient.post("/servers/list", { limit, offset_ip, ...filters });
};

export const fetchServerPlayers = (server_ip: string) => {
    return apiClient.post("/players/list", { server_ip });
};

export const verifyAuth = () => {
    return apiClient.post("/auth/validate", null);
}

export const fetchStats = () => {
    return apiClient.post("/stats", null);
}

export const authenticate = (password: string) => {
    return apiClient.post("/auth/login", { password });
}

export const setCookieReq = (token: string) => {
    return apiClient.post("/auth/set_cookie", { token });
}

export const updateServer = (server_ip: string, checked: boolean | null, auth_me: boolean | null, crashed: boolean | null) => {
    return apiClient.post("/server/update", {server_ip, checked, auth_me, crashed});
}
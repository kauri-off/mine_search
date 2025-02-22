import apiClient from "./apiConfig";

export const fetchServerInfo = (ip: string) => {
    return apiClient.post("/server/info", {ip});
}

export const fetchServerList = (limit: number, offset_ip: string | null, license: boolean | null) => {
    return apiClient.post("/servers/list", { limit, offset_ip, license });
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


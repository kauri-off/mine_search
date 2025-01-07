import { apiClient } from "./apiConfig";

export const getServer = (ip: string) => {
    return apiClient.post("/server", {ip});
}

export const getServers = (limit: number, offset_ip: string | null) => {
    return apiClient.post("/servers", { limit, offset_ip });
};

export const getPlayers = (server_ip: string) => {
    return apiClient.post("/players", { server_ip });
};
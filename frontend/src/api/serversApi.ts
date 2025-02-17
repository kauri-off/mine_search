import apiClient from "./apiConfig";

export const getServer = (ip: string) => {
    return apiClient.post("/server", {ip}, {
        headers: {
            'use-pin': true // Флаг, чтобы axios добавил PIN
        }
    });
}

export const getServers = (limit: number, offset_ip: string | null, license: boolean | null) => {
    return apiClient.post("/servers", { limit, offset_ip, license }, {
        headers: {
            'use-pin': true // Флаг, чтобы axios добавил PIN
        }
    });
};

export const getPlayers = (server_ip: string) => {
    return apiClient.post("/players", { server_ip }, {
        headers: {
            'use-pin': true // Флаг, чтобы axios добавил PIN
        }
    });
};

export const apiCheck = (password: string) => {
    return apiClient.post("/check", null, {
        headers: {
            'x-password': password
        }
    });
}

export const isProtected = () => {
    return apiClient.post("/protected");
}

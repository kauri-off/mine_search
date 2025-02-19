import apiClient from "./apiConfig";

export const getServer = (ip: string) => {
    return apiClient.post("/server", {ip}, {
        headers: {
            'use-auth': true
        }
    });
}

export const getServers = (limit: number, offset_ip: string | null, license: boolean | null) => {
    return apiClient.post("/servers", { limit, offset_ip, license }, {
        headers: {
            'use-auth': true
        }
    });
};

export const getPlayers = (server_ip: string) => {
    return apiClient.post("/players", { server_ip }, {
        headers: {
            'use-auth': true
        }
    });
};

export const auth = (password: string) => {
    return apiClient.post("/auth", { password });
}



export const checkAuth = () => {
    return apiClient.post("/check_auth", null, {
        headers: {
            'use-auth': true
        }
    });
}


import apiClient from "./apiConfig";

export const fetchServerInfo = (ip: string) => {
    return apiClient.post("/server/info", {ip}, {
        headers: {
            'use-auth': true
        }
    });
}

export const fetchServerList = (limit: number, offset_ip: string | null, license: boolean | null) => {
    return apiClient.post("/servers/list", { limit, offset_ip, license }, {
        headers: {
            'use-auth': true
        }
    });
};

export const fetchServerPlayers = (server_ip: string) => {
    return apiClient.post("/players/list", { server_ip }, {
        headers: {
            'use-auth': true
        }
    });
};

export const authenticate = (password: string) => {
    return apiClient.post("/auth/login", { password });
}



export const verifyAuth = () => {
    return apiClient.post("/auth/validate", null, {
        headers: {
            'use-auth': true
        }
    });
}


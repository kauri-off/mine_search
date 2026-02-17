export interface ServerResponse {
    id: number;
    ip: string;
    online: number;
    max: number;
    version_name: string;
    protocol: number;
    license: boolean;
    disconnect_reason_html?: string | null;
    updated: string; // ISO String from DateTime<Utc>
    description_html: string;
    was_online: boolean;
    unique_players: number;
    checked: boolean;
    spoofable?: boolean | null;
    crashed: boolean;
}

export interface ServerListRequest {
    limit: number;
    offset_id?: number | null;
    licensed?: boolean | null;
    checked?: boolean | null;
    spoofable?: boolean | null;
    crashed?: boolean | null;
    has_players?: boolean | null;
    online?: boolean | null;
}

export interface ServerDataRequest {
    server_id: number;
    limit: number;
}

export interface DataResponse {
    server_id: number;
    online: number;
    max: number;
    players: string[];
    timestamp: string;
}

export interface StatsReturn {
    total_servers: number;
    cracked_servers: number;
}

export interface UpdateServerBody {
    server_ip: string;
    checked?: boolean;
    spoofable?: boolean | null;
    crashed?: boolean;
}

export interface AuthBody {
    password: string;
}

export interface AuthReturn {
    token: string;
}
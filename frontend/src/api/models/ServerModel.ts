export interface ServerModel {
    ip: string,
    online: number,
    max: number,
    version_name: string,
    protocol: number,
    license: boolean,
    white_list: boolean | null,
    description: any,
    description_html: string,
    last_seen: string,
    player_count: number,
    was_online: boolean,
    checked: boolean,
    auth_me: boolean | null,
    crashed: boolean
}
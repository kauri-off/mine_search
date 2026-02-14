export interface ServerModel {
    ip: string,
    online: number,
    max: number,
    version_name: string,
    protocol: number,
    license: boolean,
    white_list: boolean | null,
    updated: string,
    description: any,
    description_html: string,
    was_online: boolean,
    checked: boolean,
    auth_me: boolean | null,
    crashed: boolean
}
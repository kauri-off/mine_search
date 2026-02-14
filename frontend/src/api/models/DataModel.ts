export interface DataModel {
    id: number,
    server_id: number,
    online: number,
    max: number
    players: [Players]
    timestamp: string,
}

export interface Players {
    uuid: string,
    name: string
}
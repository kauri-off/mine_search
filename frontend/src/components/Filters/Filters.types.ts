export interface FiltersList {
    licensed: boolean | null,
    has_players: boolean | null
}

export interface FiltersProps {
    callback: (filters: FiltersList) => void;
}
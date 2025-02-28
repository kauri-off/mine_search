export interface FiltersList {
    licensed: boolean | null,
    has_players: boolean | null,
    white_list: boolean | null,
    was_online: boolean | null
}

export interface FiltersProps {
    filters: FiltersList,
    setFilters: React.Dispatch<React.SetStateAction<FiltersList>>
}
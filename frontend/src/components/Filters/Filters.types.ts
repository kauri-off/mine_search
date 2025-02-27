export interface FiltersList {
    licensed: boolean | null,
    has_players: boolean | null
}

export interface FiltersProps {
    filters: FiltersList,
    setFilters: React.Dispatch<React.SetStateAction<FiltersList>>
}
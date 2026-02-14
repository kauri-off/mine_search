export interface FiltersList {
    licensed: boolean | null,
    white_list: boolean | null,
    checked: boolean | null,
    auth_me: boolean | null,
    crashed: boolean | null
}

export interface FiltersProps {
    filters: FiltersList,
    setFilters: React.Dispatch<React.SetStateAction<FiltersList>>
}
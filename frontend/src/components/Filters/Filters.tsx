import { FiltersList, FiltersProps } from "./Filters.types";

const FilterButton = ({
  label,
  value,
  onClick,
  colors,
}: {
  label: string;
  value: boolean | null;
  onClick: () => void;
  colors: Record<"true" | "false" | "null", string>;
}) => {
  const key = value === null ? "null" : value ? "true" : "false";
  return (
    <button className={`btn btn-sm ${colors[key]}`} onClick={onClick}>
      {label}: {value === null ? "Any" : value ? "Yes" : "No"}
    </button>
  );
};

function Filters({ filters, setFilters }: FiltersProps) {
  const cycleFilter = (key: keyof FiltersList) => {
    setFilters((prev) => ({
      ...prev,
      [key]: prev[key] === null ? true : prev[key] === true ? false : null,
    }));
  };

  const filterConfigs: {
    key: keyof FiltersList;
    label: string;
    colors: Record<"true" | "false" | "null", string>;
  }[] = [
    {
      key: "licensed" as keyof FiltersList,
      label: "LICENSED",
      colors: {
        true: "btn-danger",
        false: "btn-success",
        null: "btn-secondary",
      },
    },
    {
      key: "has_players" as keyof FiltersList,
      label: "HAS PLAYERS",
      colors: {
        true: "btn-success",
        false: "btn-danger",
        null: "btn-secondary",
      },
    },
    {
      key: "white_list" as keyof FiltersList,
      label: "WHITE LIST",
      colors: {
        true: "btn-danger",
        false: "btn-success",
        null: "btn-secondary",
      },
    },
    {
      key: "was_online" as keyof FiltersList,
      label: "WAS ONLINE",
      colors: {
        true: "btn-success",
        false: "btn-danger",
        null: "btn-secondary",
      },
    },
  ];

  return (
    <div className="row mb-3 g-3">
      {filterConfigs.map(({ key, label, colors }) => (
        <div className="col-auto" key={key}>
          <FilterButton
            label={label}
            value={filters[key]}
            onClick={() => cycleFilter(key)}
            colors={colors}
          />
        </div>
      ))}
    </div>
  );
}

export default Filters;

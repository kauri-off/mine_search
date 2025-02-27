import { FiltersList, FiltersProps } from "./Filters.types";

function Filters({ filters, setFilters }: FiltersProps) {
  const cycleFilter = (key: keyof FiltersList) => {
    setFilters((prev) => ({
      ...prev,
      [key]: prev[key] === null ? true : prev[key] === true ? false : null,
    }));
  };

  return (
    <div className="row mb-3">
      {(["licensed", "has_players"] as (keyof FiltersList)[]).map((key) => (
        <div className="col-auto" key={key}>
          <button
            className={`btn btn-sm ${
              filters[key] === null
                ? "btn-secondary"
                : filters[key]
                ? "btn-success"
                : "btn-danger"
            }`}
            onClick={() => cycleFilter(key)}
          >
            {key.replace("_", " ").toUpperCase()}:{" "}
            {filters[key] === null ? "Any" : filters[key] ? "Yes" : "No"}
          </button>
        </div>
      ))}
    </div>
  );
}

export default Filters;

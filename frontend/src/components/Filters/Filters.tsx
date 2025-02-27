import { useState } from "react";
import { FiltersList, FiltersProps } from "./Filters.types";

function Filters({ callback }: FiltersProps) {
  const [filters, setFilters] = useState<FiltersList>({
    licensed: null,
    has_players: null,
  });

  const toggleFilter = (key: keyof FiltersList) => {
    setFilters((prev) => {
      const newValue =
        prev[key] === null ? true : prev[key] === true ? false : null;
      return { ...prev, [key]: newValue };
    });
  };

  return (
    <>
      <div className="row mb-3">
        <div className="col-auto">
          <button
            className={`btn btn-sm ${
              filters.licensed
                ? "btn-success"
                : filters.licensed === false
                ? "btn-danger"
                : "btn-secondary"
            }`}
            onClick={() => toggleFilter("licensed")}
          >
            Licensed:{" "}
            {filters.licensed === null
              ? "Any"
              : filters.licensed
              ? "Yes"
              : "No"}
          </button>
        </div>
        <div className="col">
          <button
            className={`btn btn-sm ${
              filters.has_players
                ? "btn-success"
                : filters.has_players === false
                ? "btn-danger"
                : "btn-secondary"
            }`}
            onClick={() => toggleFilter("has_players")}
          >
            Has players:{" "}
            {filters.has_players === null
              ? "Any"
              : filters.has_players
              ? "Yes"
              : "No"}
          </button>
        </div>
      </div>
      <div className="row mb-3">
        <div className="col-auto">
          <button className="btn btn-primary" onClick={() => callback(filters)}>
            Apply
          </button>
        </div>
      </div>
    </>
  );
}

export default Filters;

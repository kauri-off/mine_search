import { useState } from "react";
import { NavBarProps, Page } from "./NavBar.types";

function NavBar({ page }: NavBarProps) {
  const [isNavOpen, setIsNavOpen] = useState(false);

  const toggleNavbar = () => {
    setIsNavOpen(!isNavOpen);
  };

  return (
    <nav className="navbar navbar-expand-lg bg-body-tertiary mb-2">
      <div className="container-fluid">
        <a className="navbar-brand" href="/">
          MC Search
        </a>
        <button
          className="navbar-toggler"
          type="button"
          onClick={toggleNavbar}
          data-bs-toggle="collapse"
          data-bs-target="#navbarNav"
          aria-controls="navbarNav"
          aria-expanded={isNavOpen ? "true" : "false"}
          aria-label="Toggle navigation"
        >
          <span className="navbar-toggler-icon"></span>
        </button>
        <div
          className={`collapse navbar-collapse ${isNavOpen ? "show" : ""}`}
          id="navbarNavAltMarkup"
        >
          <div className="navbar-nav">
            <a
              className={"nav-link" + (page == Page.HOME ? " active" : "")}
              aria-current="page"
              href="/"
            >
              Home
            </a>
            <a
              className={"nav-link" + (page == Page.STATS ? " active" : "")}
              href="/stats"
            >
              Stats
            </a>
          </div>
        </div>
      </div>
    </nav>
  );
}

export default NavBar;

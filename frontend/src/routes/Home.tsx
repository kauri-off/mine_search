import { useNavigate } from "react-router-dom";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import ServerSearchBar from "../components/ServerSearchBar";
import { useEffect } from "react";
import { verifyAuth } from "../api/serversApi";
import FilterableServerList from "../components/FilterableServerList";

function Home() {
  let navigate = useNavigate();

  useEffect(() => {
    verifyAuth().catch((res) => {
      if (res.status == 401) {
        navigate("/auth");
      }
    });
  }, [navigate]);

  return (
    <>
      <NavBar page={Page.HOME} />
      <div className="container">
        <div className="row">
          <ServerSearchBar />
        </div>
        <div className="row">
          <FilterableServerList />
        </div>
      </div>
    </>
  );
}

export default Home;

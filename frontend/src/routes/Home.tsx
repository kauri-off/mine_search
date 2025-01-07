import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import ServerList from "../components/ServerList";
import ServerSearchBar from "../components/ServerSearchBar";

function Home() {
  return (
    <>
      <NavBar page={Page.HOME} />
      <div className="container">
        <ServerSearchBar />
        <ServerList />
      </div>
    </>
  );
}

export default Home;

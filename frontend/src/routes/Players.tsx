import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";

function Players() {
  return (
    <>
      <NavBar page={Page.PLAYERS} />
      <h1 className="text-center">TODO</h1>
    </>
  );
}

export default Players;

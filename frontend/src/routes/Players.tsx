import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";

function Players() {
  return (
    <>
      <NavBar page={Page.PLAYERS} />
    </>
  );
}

export default Players;

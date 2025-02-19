import { useEffect, useState } from "react";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import ServerList from "../components/ServerList";
import ServerSearchBar from "../components/ServerSearchBar";
import { auth, checkAuth } from "../api/serversApi";
import AuthField from "../components/AuthField";

function Home() {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkAuth().then(() => {
      setReady(true);
    });
  }, []);

  const callback = (text: string) => {
    auth(text)
      .then((res) => {
        localStorage.setItem("token", res.data.token);
        setReady(true);
      })
      .catch(() => {
        setError("Password is incorrect");
      });
  };

  return (
    <>
      <NavBar page={Page.HOME} />
      {ready && (
        <div className="container">
          <ServerSearchBar />
          <ServerList />
        </div>
      )}
      {!ready && (
        <>
          <AuthField callback={callback} />
          {error && <p className="text-danger">{error}</p>}
        </>
      )}
    </>
  );
}

export default Home;

import { useEffect, useState } from "react";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import ServerList from "../components/ServerList";
import ServerSearchBar from "../components/ServerSearchBar";
import { apiCheck, isProtected } from "../api/serversApi";
import AuthField from "../components/AuthField";

function Home() {
  const [usePin, setUsePin] = useState<boolean | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    isProtected().then((res) => {
      let pin = localStorage.getItem("panel_pin");
      setUsePin(res.data);

      if (!res.data) {
        setReady(true);
      } else if (pin) {
        checkPin(pin).then((t) => {
          if (t) {
            setReady(true);
          }
        });
      }
    });
  }, []);

  const checkPin = (pin: string) => {
    return apiCheck(pin)
      .then(() => true) // Если `apiCheck` успешно, возвращаем true
      .catch(() => false); // Если ошибка, возвращаем false
  };

  const callback = (text: string) => {
    checkPin(text).then((t) => {
      if (t) {
        localStorage.setItem("panel_pin", text);
        setReady(true);
      }
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
      {usePin && !ready && <AuthField callback={callback} />}
    </>
  );
}

export default Home;

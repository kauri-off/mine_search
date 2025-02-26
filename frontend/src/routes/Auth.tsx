import { useEffect, useState } from "react";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import { useLocation, useNavigate } from "react-router-dom";
import { authenticate, setCookieReq, verifyAuth } from "../api/serversApi";
import AuthField from "../components/AuthField";

function Auth() {
  const [error, setError] = useState<string | null>(null);
  const location = useLocation();
  const navigate = useNavigate();
  const [url, setUrl] = useState("/");

  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const token = params.get("token");
    const back_url = params.get("back_url") ?? "/";
    setUrl(back_url);

    if (token) {
      setCookieReq(token).then(() => navigate(back_url));
    } else {
      verifyAuth().then(() => navigate(back_url));
    }
  }, [location, navigate]);

  const callback = (text: string) => {
    authenticate(text)
      .then((res) => {
        localStorage.setItem("token", res.data.token);
        navigate(url);
      })
      .catch(() => {
        setError("Password is incorrect");
      });
  };

  return (
    <>
      <NavBar page={Page.NONE} />
      <AuthField callback={callback} />
      {error && <p className="text-danger">{error}</p>}
    </>
  );
}

export default Auth;

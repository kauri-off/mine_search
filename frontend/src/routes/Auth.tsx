import { useEffect, useState } from "react";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import { useLocation, useNavigate } from "react-router-dom";
import { authenticate, verifyAuth } from "../api/serversApi";
import AuthField from "../components/AuthField";

function Auth() {
  const [error, setError] = useState<string | null>(null);
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const token = params.get("token");

    if (token) {
      localStorage.setItem("token", token);
      params.delete("token");
      navigate("/");
    } else {
      verifyAuth().then(() => navigate("/"));
    }
  }, [location, navigate]);

  const callback = (text: string) => {
    authenticate(text)
      .then((res) => {
        localStorage.setItem("token", res.data.token);
        navigate("/");
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

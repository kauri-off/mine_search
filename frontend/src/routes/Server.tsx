import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { PlayerModel } from "../api/models/PlayerModel";
import { ServerModel } from "../api/models/ServerModel";
import { fetchServerPlayers, fetchServerInfo } from "../api/serversApi";
import Loading from "../components/Loading";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import PlayersTable from "../components/PlayersTable";
import ServerTable from "../components/ServerTable";

function Server() {
  const { ip } = useParams();

  const [server, setServer] = useState<ServerModel | null>(null);
  const [players, setPlayers] = useState<PlayerModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    fetchServerInfo(ip!)
      .then((res) => {
        setServer(res.data);
      })
      .catch((res) => {
        setLoading(false);
        setError(true);

        if (res.status == 401) {
          navigate("/auth");
        }
      });
  }, [navigate]);

  useEffect(() => {
    if (!server) return;

    fetchServerPlayers(server.ip).then((res) => {
      setPlayers(res.data);
      setLoading(false);
    });
  }, [server]);

  if (error) {
    return (
      <>
        <NavBar page={Page.NONE} />
        <h1>Error</h1>
      </>
    );
  }

  return (
    <>
      <NavBar page={Page.NONE} />
      {loading ? (
        <Loading />
      ) : (
        <>
          <ServerTable server={server!} /> <PlayersTable players={players} />
        </>
      )}
    </>
  );
}

export default Server;

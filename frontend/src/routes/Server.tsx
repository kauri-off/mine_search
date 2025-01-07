import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { PlayerModel } from "../api/models/PlayerModel";
import { ServerModel } from "../api/models/ServerModel";
import { getPlayers, getServer } from "../api/serversApi";
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

  useEffect(() => {
    getServer(ip!).then((res) => {
      setServer(res.data);
    });
  }, []);

  useEffect(() => {
    if (!server) return;

    getPlayers(server.ip).then((res) => {
      setPlayers(res.data);
      setLoading(false);
    });
  }, [server]);

  return (
    <>
      <NavBar page={Page.NONE} />
      {loading ? <Loading /> : <ServerTable server={server!} />}
      {!loading && <PlayersTable players={players} />}
    </>
  );
}

export default Server;

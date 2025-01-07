import { useEffect, useState } from "react";
import { ServerModel } from "../../api/models/ServerModel";
import { getServers } from "../../api/serversApi";
import Server from "../ServerCard";
import { useInView } from "react-intersection-observer";
import Loading from "../Loading";

function ServerList() {
  const [servers, setServers] = useState<ServerModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [ref, inView] = useInView();

  const fetchServers = async () => {
    if (loading || !hasMore) return;

    setLoading(true);

    try {
      const lastServerIp =
        servers.length > 0 ? servers[servers.length - 1].ip : null;
      const res = await getServers(10, lastServerIp);

      if (res.data.length === 0) {
        setHasMore(false);
      } else {
        setServers((prev) => [...prev, ...res.data]);
      }
    } catch (error) {
      console.error("Ошибка загрузки серверов:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (inView && hasMore) {
      fetchServers();
    }
  }, [inView]);

  return (
    <>
      {servers.map((server) => (
        <Server key={server.ip} server={server} />
      ))}
      {loading && <Loading />}
      {!hasMore && <p>Больше серверов нет.</p>}
      <div ref={ref} style={{ height: "1px" }}></div>
    </>
  );
}

export default ServerList;

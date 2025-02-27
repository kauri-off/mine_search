import { useEffect, useState } from "react";
import { ServerModel } from "../../api/models/ServerModel";
import { fetchServerList } from "../../api/serversApi";
import ServerCard from "../ServerCard";
import { useInView } from "react-intersection-observer";
import Loading from "../Loading";
import Filters from "../Filters";
import { FiltersList } from "../Filters/Filters.types";

function FilterableServerList() {
  const [servers, setServers] = useState<ServerModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [ref, inView] = useInView();
  const [filter, setFilter] = useState<FiltersList>({
    licensed: null,
    has_players: null,
  });

  const fetchServers = async () => {
    if (loading || !hasMore) return;

    setLoading(true);

    try {
      const lastServerIp =
        servers.length > 0 ? servers[servers.length - 1].ip : null;
      const res = await fetchServerList(
        18,
        lastServerIp,
        filter.licensed,
        filter.has_players
      );

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

  useEffect(() => {
    setServers([]);
    fetchServers();
  }, [filter]);

  return (
    <>
      <div className="container">
        <Filters callback={setFilter} />

        <div className="row row-cols-1 row-cols-sm-1 row-cols-lg-2 row-cols-xxl-3">
          {servers.map((server) => (
            <div className="col" key={server.ip}>
              <ServerCard server={server} />
            </div>
          ))}
        </div>
      </div>

      {loading && <Loading />}
      {!hasMore && <p>No servers left.</p>}
      <div ref={ref} style={{ height: "1px" }}></div>
    </>
  );
}

export default FilterableServerList;

import { useEffect, useState } from "react";
import { ServerModel } from "../../api/models/ServerModel";
import { fetchServerList } from "../../api/serversApi";
import ServerCard from "../ServerCard";
import { useInView } from "react-intersection-observer";
import Loading from "../Loading";
import Filters from "../Filters";
import { FiltersList } from "../Filters/Filters.types";

const LOCAL_STORAGE_KEY = "server_filters";

function FilterableServerList() {
  const [servers, setServers] = useState<ServerModel[]>([]);

  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [ref, inView] = useInView();

  const [filters, setFilters] = useState<FiltersList>(() => {
    try {
      const savedFilters = localStorage.getItem(LOCAL_STORAGE_KEY);
      return savedFilters ? JSON.parse(savedFilters) : getDefaultFilters();
    } catch (_) {
      return getDefaultFilters();
    }
  });

  function getDefaultFilters(): FiltersList {
    return {
      licensed: null,
      has_players: null,
      white_list: null,
      was_online: null,
      checked: null,
      auth_me: null,
      crashed: null,
    };
  }

  useEffect(() => {
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(filters));
    fetchServers(true);
  }, [filters]);

  const fetchServers = async (reset = false) => {
    if (loading || (!reset && !hasMore)) return;

    setLoading(true);

    try {
      const lastServerIp = reset ? null : servers[servers.length - 1]?.ip;
      const res = await fetchServerList(18, lastServerIp, filters);

      setServers((prev) => (reset ? res.data : [...prev, ...res.data]));
      setHasMore(res.data.length > 0);
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
      <div className="container">
        <Filters filters={filters} setFilters={setFilters} />
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
      {!loading && <div ref={ref} style={{ height: "1px" }}></div>}
    </>
  );
}

export default FilterableServerList;

import { useEffect, useState } from "react";
import NavBar from "../components/NavBar";
import { Page } from "../components/NavBar/NavBar.types";
import Loading from "../components/Loading";
import { StatsModel } from "../api/models/StatsModel";
import { fetchStats } from "../api/serversApi";

function Stats() {
  const [loading, setLoading] = useState(true);
  const [stats, setStats] = useState<StatsModel | null>(null);

  useEffect(() => {
    fetchStats().then((res) => {
      setLoading(false);
      setStats(res.data);
    });
  }, []);

  return (
    <>
      <NavBar page={Page.STATS} />
      {loading ? (
        <Loading />
      ) : (
        <div className="container mt-4">
          <h2 className="text-center">Statistics</h2>
          <div className="row justify-content-center">
            <div className="col-md-4">
              <div className="card text-white bg-primary mb-3">
                <div className="card-header">Total Servers</div>
                <div className="card-body">
                  <h5 className="card-title">{stats!.total_servers}</h5>
                </div>
              </div>
            </div>
            <div className="col-md-4">
              <div className="card text-white bg-danger mb-3">
                <div className="card-header">Cracked Servers</div>
                <div className="card-body">
                  <h5 className="card-title">{stats!.cracked_servers}</h5>
                </div>
              </div>
            </div>
            <div className="col-md-4">
              <div className="card text-white bg-success mb-3">
                <div className="card-header">Players</div>
                <div className="card-body">
                  <h5 className="card-title">{stats!.players}</h5>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

export default Stats;

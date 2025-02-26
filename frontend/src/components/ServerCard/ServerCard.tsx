import { ServerProps } from "./ServerCard.types";
import ServerStatus from "../ServerStatus";

function ServerCard({ server }: ServerProps) {
  let date = new Date(server.last_seen + "Z");
  const formattedDate = new Intl.DateTimeFormat("en-US", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(date);

  return (
    <div className="card mb-3 border-secondary shadow">
      <div className="card-header d-flex align-items-center">
        <span className="me-3">
          <strong>{server.player_count}</strong> / {server.max}
        </span>
        <ServerStatus server={server} />
        <small className="text-muted ms-auto">{server.ip}</small>
      </div>

      <div className="card-body">
        <h5 className="card-title">Version: {server.version_name}</h5>
        <p
          className="card-text bg-dark p-1 rounded"
          dangerouslySetInnerHTML={{ __html: server.description_html }}
        ></p>
        <div className="d-flex flex-wrap gap-3 mb-3">
          <span
            className={`badge ${
              server.license ? "bg-danger" : "bg-success"
            } w-auto`}
          >
            {server.license ? "Licensed" : "Unlicensed"}
          </span>
          <span
            className={`badge ${
              server.white_list ? "bg-danger" : "bg-success"
            } w-auto`}
          >
            {server.white_list ? "Whitelist: Yes" : "Whitelist: No"}
          </span>
          <span className="badge bg-secondary w-auto">
            Last seen: {formattedDate}
          </span>
        </div>

        <a
          href={`/server/${server.ip}`}
          className="btn btn-primary w-100"
          target="_blank"
          rel="noopener noreferrer"
        >
          Open Server
        </a>
      </div>
    </div>
  );
}

export default ServerCard;

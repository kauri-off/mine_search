import ServerStatus from "../ServerStatus";
import { ServerProps } from "./ServerCard.types";

function Server({ server }: ServerProps) {
  return (
    <div className="card mb-4">
      <h5 className="card-header">
        [{server.player_count}]
        <ServerStatus server={server} />
        {" " + server.ip}
      </h5>
      <div className="card-body">
        <h5 className="card-title">Version: {server.version_name}</h5>
        <h5
          className="card-text bg-dark rounded p-2"
          dangerouslySetInnerHTML={{ __html: server.description_html }}
        ></h5>
        <a
          href={"/server/" + server.ip}
          className="btn btn-primary"
          target="_blank"
          rel="noopener noreferrer"
        >
          Open server
        </a>
      </div>
    </div>
  );
}

export default Server;

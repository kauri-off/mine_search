import { ServerStatusProps } from "./ServerStatus.types";

function ServerStatus({ server }: ServerStatusProps) {
  let date = new Date(server.last_seen);
  let now = new Date(Date.now());

  let lastSeenMin = (now.getTime() - date.getTime()) / 1000 / 60 - 3 * 60;

  return (
    <span>
      {lastSeenMin < 15 ? (
        <span className="badge text-bg-success">online</span>
      ) : (
        <span className="badge text-bg-danger">offline</span>
      )}
    </span>
  );
}

export default ServerStatus;

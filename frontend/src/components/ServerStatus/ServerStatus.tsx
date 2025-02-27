import { ServerStatusProps } from "./ServerStatus.types";

function ServerStatus({ server }: ServerStatusProps) {
  return server.was_online ? (
    <span className="badge text-bg-success">online</span>
  ) : (
    <span className="badge text-bg-danger">offline</span>
  );
}

export default ServerStatus;

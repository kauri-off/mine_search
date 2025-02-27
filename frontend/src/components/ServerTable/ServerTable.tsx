import ServerStatus from "../ServerStatus";
import ServerTableProps from "./ServerTable.types";

function ServerTable({ server }: ServerTableProps) {
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
    <div className="col">
      <ul className="list-group">
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>Status</strong>
          <span>
            <ServerStatus server={server} />
          </span>
        </li>
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>IP</strong>
          <span>{server.ip}</span>
        </li>
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>Last Seen</strong>
          <span>{formattedDate}</span>
        </li>
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>Online</strong>
          <span>
            {server.online} / {server.max}
          </span>
        </li>
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>Version</strong>
          <span>{server.version_name}</span>
        </li>
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>License</strong>
          <span>{server.license ? "Yes" : "No"}</span>
        </li>
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>White List</strong>
          <span>
            {server.white_list
              ? server.white_list!
                ? "Enabled"
                : "Disabled"
              : "None"}
          </span>
        </li>
      </ul>
      <div
        className="bg-black p-3 my-3 text-light rounded"
        dangerouslySetInnerHTML={{ __html: server.description_html }}
      ></div>
    </div>
  );
}

export default ServerTable;

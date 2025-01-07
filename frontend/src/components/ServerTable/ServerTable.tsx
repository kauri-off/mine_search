import ServerTableProps from "./ServerTable.types";

function ServerTable({ server }: ServerTableProps) {
  return (
    <div className="p-4">
      <ul className="list-group">
        <li className="list-group-item d-flex">
          <strong style={{ width: 150 }}>IP</strong>
          <span>{server.ip}</span>
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
        className="bg-dark p-3 mt-3 text-light rounded"
        dangerouslySetInnerHTML={{ __html: server.description_html }}
      ></div>
    </div>
  );
}

export default ServerTable;

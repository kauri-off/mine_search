import { PlayersProps } from "./PlayersTable.types";

function PlayersTable({ players }: PlayersProps) {
  return (
    <div className="p-4">
      <table className="table table-striped table-hover">
        <thead className="table-dark">
          <tr>
            <th>Name</th>
            <th>Last Seen</th>
          </tr>
        </thead>
        <tbody>
          {players.map((player) => {
            let date = new Date(player.last_seen);
            const formattedDate = new Intl.DateTimeFormat("en-US", {
              year: "numeric",
              month: "2-digit",
              day: "2-digit",
              hour: "2-digit",
              minute: "2-digit",
              second: "2-digit",
            }).format(date);
            return (
              <tr key={player.id}>
                <td>{player.name}</td>
                <td>{formattedDate}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export default PlayersTable;

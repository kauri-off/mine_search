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
          {players.map((player) => (
            <tr key={player.id}>
              <td>{player.name}</td>
              <td>{player.last_seen}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default PlayersTable;

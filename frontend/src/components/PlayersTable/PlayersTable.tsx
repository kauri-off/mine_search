import { PlayersProps } from "./PlayersTable.types";

function PlayersList({ players }: PlayersProps) {
  return (
    <div className="col">
      <div className="list-group">
        {players.map((player) => {
          let date = new Date(player.last_seen + "Z");
          const formattedDate = new Intl.DateTimeFormat("en-US", {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          }).format(date);
          return (
            <div
              key={player.id}
              className="list-group-item list-group-item-action d-flex justify-content-between align-items-center"
            >
              <span className="fw-bold">{player.name}</span>
              <span className="text-muted">{formattedDate}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default PlayersList;

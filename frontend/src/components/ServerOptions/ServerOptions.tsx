import { ServerModel } from "../../api/models/ServerModel";
import { updateServer } from "../../api/serversApi";
import { ServerOptionsProps } from "./ServerOptions.types";

const OptionButton = ({
  label,
  value,
  onClick,
}: {
  label: string;
  value: boolean | null;
  onClick: () => void;
}) => {
  const color = value === null ? "secondary" : value ? "success" : "danger";
  return (
    <button className={`btn btn-sm btn-${color}`} onClick={onClick}>
      {label}
    </button>
  );
};

function ServerOptions({ server, setServer }: ServerOptionsProps) {
  const cycleFilter = (key: keyof ServerModel) => {
    const new_value =
      server[key] === null ? true : server[key] === true ? false : true;
    updateServer(
      server.ip,
      key === "checked" ? new_value : server.checked,
      key === "auth_me" ? new_value : server.auth_me,
      key === "crashed" ? new_value : server.crashed
    ).then(() => {
      setServer((prev) => ({ ...prev, [key]: new_value } as ServerModel));
    });
  };

  const optionConfigs: {
    key: keyof ServerModel;
    label: string;
  }[] = [
    {
      key: "checked" as keyof ServerModel,
      label: "CHECKED",
    },
    {
      key: "auth_me" as keyof ServerModel,
      label: "AUTH ME",
    },
    {
      key: "crashed" as keyof ServerModel,
      label: "CRASHED",
    },
  ];

  return (
    <div className="row mb-3 g-3">
      {optionConfigs.map(({ key, label }) => (
        <div className="col-auto" key={key}>
          <OptionButton
            label={label}
            value={server[key]}
            onClick={() => cycleFilter(key)}
          />
        </div>
      ))}
    </div>
  );
}

export default ServerOptions;

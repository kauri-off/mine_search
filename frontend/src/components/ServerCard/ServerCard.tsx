import { ServerProps } from "./ServerCard.types";

function Server({ server }: ServerProps) {
  return (
    <div className="card mb-4">
      <h5 className={"card-header" + (server.license ? "" : " text-success")}>
        {server.ip}
      </h5>
      <div className="card-body">
        <h5
          className="card-title bg-dark rounded p-2"
          dangerouslySetInnerHTML={{ __html: server.description_html }}
        ></h5>
        <a href={"/server/" + server.ip} className="btn btn-primary">
          Open server
        </a>
      </div>
    </div>
  );
}

export default Server;

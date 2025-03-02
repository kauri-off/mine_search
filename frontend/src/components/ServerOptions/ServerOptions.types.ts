import { ServerModel } from "../../api/models/ServerModel";

export interface ServerOptionsProps {
    server: ServerModel,
    setServer: React.Dispatch<React.SetStateAction<ServerModel | null>>
}
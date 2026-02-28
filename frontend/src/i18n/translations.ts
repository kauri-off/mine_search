import type { Locale } from "date-fns";

export interface Translations {
  dateFnsLocale: Locale;
  dashboard: {
    title: string;
    all: string;
    cracked: string;
    loaded: string;
  };
  filters: {
    label: string;
    fields: {
      licensed: string;
      checked: string;
      spoofable: string;
      crashed: string;
      has_players: string;
      online: string;
      is_forge: string;
      has_none_players: string;
    };
    reset: string;
    triState: {
      all: string;
      yes: string;
      no: string;
    };
  };
  addIp: {
    label: string;
    add: string;
    adding: string;
    invalidFormat: string;
    error: string;
  };
  serverGrid: {
    loading: string;
    empty: string;
    end: string;
  };
  serverDetail: {
    back: string;
    loading: string;
    notFound: string;
    disconnectReason: string;
  };
  serverInfo: {
    status: string;
    onlineCount: string;
    statusOnline: string;
    statusOffline: string;
    licensed: string;
    forgeModded: string;
    lastSeen: string;
    yes: string;
    no: string;
    management: string;
    checked: string;
    spoofable: string;
    crashed: string;
    ping: string;
    ms: string;
    reloadingIn: (n: number) => string;
    choosePingType: string;
    withConnection: string;
    withoutConnection: string;
    pingServer: string;
    deleteConfirm: (ip: string) => string;
    deleteWarning: string;
    cancel: string;
    deleting: string;
    confirm: string;
    deleteServer: string;
  };
  playersTable: {
    title: string;
    name: string;
    status: string;
    empty: string;
  };
  onlineGraph: {
    title: string;
  };
  login: {
    title: string;
    token: string;
    login: string;
    wrongPassword: string;
    networkError: string;
    tooManyRequests: string;
  };
}

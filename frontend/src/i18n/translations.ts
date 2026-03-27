import type { Locale } from "date-fns";

export interface Translations {
  dateFnsLocale: Locale;
  dashboard: {
    title: string;
    all: string;
    cracked: string;
    loaded: string;
    addTarget: string;
    bulkImport: string;
    searchPlaceholder: string;
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
  bulkImport: {
    title: string;
    placeholder: string;
    parse: string;
    importN: (n: number) => string;
    importing: string;
    success: (n: number) => string;
    error: string;
    invalidLines: (n: number) => string;
    quickMode: string;
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
    editServer: string;
    saveChanges: string;
    cancelEdit: string;
    port: string;
    protocol: string;
    versionName: string;
    isOnline: string;
    favicon: string;
    editSuccess: string;
    editError: string;
  };
  playersTable: {
    title: string;
    name: string;
    status: string;
    empty: string;
    deletePlayer: string;
    deleteConfirm: string;
    deleteYes: string;
    deleteNo: string;
    lastSeen: string;
  };
  onlineGraph: {
    title: string;
    online: string;
  };
  stats: {
    title: string;
    back: string;
    totalServers: string;
    online: string;
    cracked: string;
    crashed: string;
    forge: string;
    spoofable: string;
    totalPlayers: string;
    adminPlayers: string;
    avgPing: string;
    dbSize: string;
    faviconSize: string;
    licensedVsCracked: string;
    onlineVsOffline: string;
    playerBreakdown: string;
    topVersions: string;
    licensed: string;
    offline: string;
    admin: string;
    other: string;
    maintenance: string;
    cleanSnapshots: string;
    cleanFavicons: string;
    cleaning: string;
    cleanedRows: (n: number) => string;
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

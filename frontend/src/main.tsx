import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";

// When a new frontend version is deployed, the chunk hashes referenced by an
// already-open tab no longer exist on the server. Vite fires `vite:preloadError`
// when such a lazy import fails; reload once to pick up the fresh index.html and
// its current chunks. The timestamp guard prevents an infinite reload loop if a
// chunk is genuinely unreachable (e.g. the user is offline).
window.addEventListener("vite:preloadError", () => {
  const key = "vite:lastPreloadReload";
  const last = Number(sessionStorage.getItem(key) ?? 0);
  if (Date.now() - last > 10_000) {
    sessionStorage.setItem(key, String(Date.now()));
    window.location.reload();
  }
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

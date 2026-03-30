import { lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { LanguageProvider } from "./i18n";
import { Login } from "./pages/Login";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AppShell } from "./components/AppShell";
import { Spinner } from "./components";

const Dashboard = lazy(() =>
  import("./pages/Dashboard").then((m) => ({ default: m.Dashboard })),
);
const ServerDetail = lazy(() =>
  import("./pages/ServerDetail").then((m) => ({ default: m.ServerDetail })),
);
const Stats = lazy(() =>
  import("./pages/Stats").then((m) => ({ default: m.Stats })),
);
const Players = lazy(() =>
  import("./pages/Players").then((m) => ({ default: m.Players })),
);

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

const fallback = (
  <div className="flex-1 flex items-center justify-center">
    <Spinner className="w-10 h-10" />
  </div>
);

function App() {
  return (
    <LanguageProvider>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <div className="min-h-screen bg-[#0a0a0f] text-slate-100 font-sans">
            <ErrorBoundary>
              <Routes>
                <Route path="/login" element={<Login />} />
                <Route
                  path="/*"
                  element={
                    <ProtectedRoute>
                      <AppShell>
                        <Suspense fallback={fallback}>
                          <Routes>
                            <Route path="/" element={<Dashboard />} />
                            <Route path="/server/:ip" element={<ServerDetail />} />
                            <Route path="/stats" element={<Stats />} />
                            <Route path="/players" element={<Players />} />
                          </Routes>
                        </Suspense>
                      </AppShell>
                    </ProtectedRoute>
                  }
                />
              </Routes>
            </ErrorBoundary>
          </div>
        </BrowserRouter>
      </QueryClientProvider>
    </LanguageProvider>
  );
}

export default App;

import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { LanguageProvider } from "./i18n";
import { Login } from "./pages/Login";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AppShell } from "./components/AppShell";
import { Dashboard } from "./pages/Dashboard";
import { ServerDetail } from "./pages/ServerDetail";
import { Stats } from "./pages/Stats";
import { Players } from "./pages/Players";
import { Workers } from "./pages/Workers";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

function App() {
  return (
    <LanguageProvider>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <div className="min-h-screen bg-app text-slate-100 font-sans">
            <ErrorBoundary>
              <Routes>
                <Route path="/login" element={<Login />} />
                <Route
                  path="/*"
                  element={
                    <ProtectedRoute>
                      <AppShell>
                        <Routes>
                          <Route path="/" element={<Dashboard />} />
                          <Route path="/server/:ip" element={<ServerDetail />} />
                          <Route path="/stats" element={<Stats />} />
                          <Route path="/players" element={<Players />} />
                          <Route path="/workers" element={<Workers />} />
                        </Routes>
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

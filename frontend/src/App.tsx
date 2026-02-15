import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Login } from './pages/Login';
import { Dashboard } from './pages/Dashboard';
import { ServerDetail } from './pages/ServerDetail';

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
        <QueryClientProvider client={queryClient}>
            <BrowserRouter>
                <div className="min-h-screen bg-gray-900 text-gray-100 font-sans">
                    <Routes>
                        <Route path="/login" element={<Login />} />
                        <Route 
                            path="/" 
                            element={<Dashboard />} 
                        />
                        <Route 
                            path="/server/:ip" 
                            element={<ServerDetail />} 
                        />
                    </Routes>
                </div>
            </BrowserRouter>
        </QueryClientProvider>
    );
}

export default App;
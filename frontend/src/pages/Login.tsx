import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { authApi } from '../api/client';

export const Login = () => {
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const navigate = useNavigate();

    const handleLogin = async (e: React.ChangeEvent) => {
        e.preventDefault();
        setError('');
        
        try {
            await authApi.login({ password });
            navigate('/');
        } catch (err: any) {
            if (err.response?.status === 401) {
                setError('Неверный пароль');
            } else {
                setError('Ошибка сервера. Попробуйте позже');
            }
        }
    };

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-900 text-white">
            <form onSubmit={handleLogin} className="w-full max-w-sm p-8 bg-gray-800 rounded-lg shadow-lg">
                <h2 className="text-2xl font-bold mb-6 text-center">Server Admin</h2>
                
                {error && (
                    <div className="mb-4 p-2 bg-red-500/20 border border-red-500 text-red-500 text-sm rounded text-center">
                        {error}
                    </div>
                )}

                <div className="mb-4">
                    <label className="block mb-2 text-sm font-medium text-gray-400">Пароль доступа</label>
                    <input
                        type="password"
                        autoFocus
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        className="w-full p-2.5 bg-gray-700 border border-gray-600 rounded focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all"
                    />
                </div>
                
                <button
                    type="submit"
                    className="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 rounded transition shadow-md active:transform active:scale-95"
                >
                    Войти
                </button>
            </form>
        </div>
    );
};
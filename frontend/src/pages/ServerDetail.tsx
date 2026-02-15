import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { serverApi } from '../api/client';
import type { UpdateServerBody } from '../types';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { format } from 'date-fns';

export const ServerDetail = () => {
    const { ip } = useParams<{ ip: string }>();
    const navigate = useNavigate();
    const queryClient = useQueryClient();

    if (!ip) return null;

    const { data: server, isLoading: isInfoLoading } = useQuery({
        queryKey: ['server', ip],
        queryFn: () => serverApi.fetchInfo(ip),
    });

    const { data: history } = useQuery({
        queryKey: ['serverData', server?.id],
        queryFn: () => serverApi.fetchData({ server_id: server!.id, limit: 100 }), // 100 последних записей
        enabled: !!server?.id,
    });

    const updateMutation = useMutation({
        mutationFn: (body: UpdateServerBody) => serverApi.update(body),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['server', ip] });
        }
    });

    const handleToggle = (field: 'checked' | 'auth_me' | 'crashed') => {
        if (!server) return;
        updateMutation.mutate({
            server_ip: server.ip,
            [field]: !server[field]
        });
    };

    if (isInfoLoading) return <div className="text-white text-center mt-20">Загрузка информации...</div>;
    if (!server) return <div className="text-white text-center mt-20">Сервер не найден</div>;

    const chartData = history?.map(d => ({
        time: d.timestamp,
        online: d.online,
        formattedTime: format(new Date(d.timestamp), 'HH:mm')
    })).reverse();

    const allPlayers = Array.from(new Set(
        history?.flatMap(h => {
             if (Array.isArray(h.players)) return h.players as string[];
             return [];
        }) || []
    ));

    return (
        <div className="p-6 max-w-7xl mx-auto text-white">
            <button onClick={() => navigate(-1)} className="mb-4 text-blue-400 hover:underline">← Назад</button>
            
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div className="lg:col-span-1 space-y-6">
                    <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
                        <h1 className="text-2xl font-bold break-all mb-2">{server.ip}</h1>
                        <p className="text-gray-400 mb-4">{server.version_name}</p>
                        
                        <div className="space-y-3">
                            <div className="flex justify-between border-b border-gray-700 pb-2">
                                <span>Статус:</span>
                                <span className={server.was_online ? "text-green-400" : "text-red-400"}>
                                    {server.was_online ? "Online" : "Offline"}
                                </span>
                            </div>
                            <div className="flex justify-between border-b border-gray-700 pb-2">
                                <span>Игроки:</span>
                                <span>{server.online} / {server.max}</span>
                            </div>
                            <div className="flex justify-between border-b border-gray-700 pb-2">
                                <span>Лицензия:</span>
                                <span>{server.license ? "Да" : "Нет"}</span>
                            </div>
                        </div>

                        <div className="mt-6 space-y-2">
                            <h3 className="font-semibold mb-2 text-gray-300">Управление:</h3>
                            <ToggleButton 
                                label="Проверен (Checked)" 
                                active={!!server.checked} 
                                onClick={() => handleToggle('checked')} 
                            />
                            <ToggleButton 
                                label="Auth Me" 
                                active={!!server.auth_me} 
                                onClick={() => handleToggle('auth_me')} 
                            />
                            <ToggleButton 
                                label="Краш (Crashed)" 
                                active={!!server.crashed} 
                                onClick={() => handleToggle('crashed')} 
                                color="red"
                            />
                        </div>
                    </div>

                    <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 overflow-hidden">
                        <h3 className="font-bold mb-4">Описание (HTML)</h3>
                        <div 
                            className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded"
                            dangerouslySetInnerHTML={{ __html: server.description_html }} 
                        />
                    </div>
                </div>

                <div className="lg:col-span-2 space-y-6">
                    <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 h-96">
                        <h3 className="font-bold mb-4">Онлайн за последнее время</h3>
                        <ResponsiveContainer width="100%" height="100%">
                            <AreaChart data={chartData}>
                                <defs>
                                    <linearGradient id="colorOnline" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                                        <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                                    </linearGradient>
                                </defs>
                                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                                <XAxis dataKey="formattedTime" stroke="#9ca3af" />
                                <YAxis stroke="#9ca3af" />
                                <Tooltip 
                                    contentStyle={{ backgroundColor: '#1f2937', border: 'none', borderRadius: '8px' }}
                                    itemStyle={{ color: '#fff' }}
                                />
                                <Area type="monotone" dataKey="online" stroke="#3b82f6" fillOpacity={1} fill="url(#colorOnline)" />
                            </AreaChart>
                        </ResponsiveContainer>
                    </div>

                    <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
                        <h3 className="font-bold mb-4">Игроки (История)</h3>
                        {allPlayers.length > 0 ? (
                            <div className="flex flex-wrap gap-2">
                                {allPlayers.map((player, idx) => (
                                    <span key={idx} className="bg-gray-700 px-2 py-1 rounded text-sm text-blue-200">
                                        {player}
                                    </span>
                                ))}
                            </div>
                        ) : (
                            <span className="text-gray-500">История игроков пуста</span>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};

const ToggleButton = ({ label, active, onClick, color = 'blue' }: any) => (
    <button
        onClick={onClick}
        className={`w-full py-2 px-4 rounded font-medium transition flex justify-between items-center
            ${active 
                ? (color === 'red' ? 'bg-red-600 hover:bg-red-700' : 'bg-blue-600 hover:bg-blue-700') 
                : 'bg-gray-700 hover:bg-gray-600'
            }`}
    >
        <span>{label}</span>
        <span className="text-xs uppercase bg-black/20 px-2 py-0.5 rounded">
            {active ? 'ON' : 'OFF'}
        </span>
    </button>
);
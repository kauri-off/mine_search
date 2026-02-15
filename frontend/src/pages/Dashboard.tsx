import { useState, useRef, useCallback } from 'react';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import type { InfiniteData } from '@tanstack/react-query';
import { serverApi } from '../api/client';
import type { ServerListRequest, ServerResponse } from '../types';
import { Link } from 'react-router-dom';
import { formatDistanceToNow } from 'date-fns';
import { enUS } from 'date-fns/locale';

export const Dashboard = () => {
    const [filters, setFilters] = useState<Omit<ServerListRequest, 'offset_id'>>({
        limit: 50,
        licensed: null,
        white_list: null,
        checked: null,
        auth_me: null,
        crashed: null,
    });

    const { data: stats } = useQuery({
        queryKey: ['stats'],
        queryFn: serverApi.fetchStats,
    });

    const {
        data,
        fetchNextPage,
        hasNextPage,
        isFetchingNextPage,
        isLoading,
    } = useInfiniteQuery<
        ServerResponse[],
        Error,
        InfiniteData<ServerResponse[]>,
        [string, Omit<ServerListRequest, 'offset_id'>],
        number | null
    >({
        queryKey: ['servers', filters],
        queryFn: async ({ pageParam = null }) => {
            return await serverApi.fetchList({ 
                ...filters, 
                offset_id: pageParam 
            });
        },
        getNextPageParam: (lastPage) => {
            if (!lastPage || lastPage.length < filters.limit) return undefined;
            return lastPage[lastPage.length - 1].id;
        },
        initialPageParam: null,
    });

    const observer = useRef<IntersectionObserver | null>(null);
    
    const lastServerRef = useCallback((node: HTMLAnchorElement) => {
        if (isLoading || isFetchingNextPage) return;
        if (observer.current) observer.current.disconnect();
        
        observer.current = new IntersectionObserver(entries => {
            if (entries[0].isIntersecting && hasNextPage) {
                fetchNextPage();
            }
        });
        
        if (node) observer.current.observe(node);
    }, [isLoading, isFetchingNextPage, hasNextPage, fetchNextPage]);

    const FilterButton = ({ label, field }: { label: string, field: keyof typeof filters }) => {
        const val = filters[field];
        let color = "bg-gray-700 text-gray-300";
        if (val === true) color = "bg-green-600 text-white";
        if (val === false) color = "bg-red-600 text-white";
        
        return (
            <button
                onClick={() => {
                    const next = val === null ? true : val === true ? false : null;
                    setFilters({ ...filters, [field]: next });
                }}
                className={`px-3 py-1 rounded text-sm font-medium transition ${color}`}
            >
                {label}: {val === null ? 'All' : val ? 'Yes' : 'No'}
            </button>
        )
    };

    return (
        <div className="p-6 max-w-7xl mx-auto text-white">
            <header className="mb-8 flex justify-between items-center">
                <h1 className="text-3xl font-bold">Server list</h1>
                {stats && (
                    <div className="flex gap-4 text-sm bg-gray-800 p-3 rounded-lg">
                        <span>All: <b className="text-blue-400">{stats.total_servers}</b></span>
                        <span>Cracked: <b className="text-orange-400">{stats.cracked_servers}</b></span>
                    </div>
                )}
            </header>

            <div className="mb-6 p-4 bg-gray-800 rounded-lg flex flex-wrap gap-4 items-center">
                <span className="text-gray-400">Filters:</span>
                <FilterButton label="Licensed" field="licensed" />
                <FilterButton label="WhiteList" field="white_list" />
                <FilterButton label="Checked" field="checked" />
                <FilterButton label="Crashed" field="crashed" />
            </div>

            {isLoading ? (
                <div className="text-center py-20">Loading...</div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {data?.pages.map((group, i) => (
                        group.map((server, index) => {
                            const isLastElement = 
                                data.pages.length === i + 1 && 
                                group.length === index + 1;

                            return (
                                <Link 
                                    ref={isLastElement ? lastServerRef : null}
                                    to={`/server/${server.ip}`} 
                                    key={server.id} 
                                    className="block p-4 bg-gray-800 hover:bg-gray-750 border border-gray-700 rounded-lg transition hover:shadow-lg hover:border-blue-500"
                                >
                                    <div className="flex justify-between items-start mb-2">
                                        <h3 className="font-bold text-lg truncate">{server.ip}</h3>
                                        <span className={`w-3 h-3 rounded-full ${server.was_online ? 'bg-green-500' : 'bg-red-500'}`}></span>
                                    </div>
                                    <div className="text-sm text-gray-400 mb-2">
                                        {server.version_name} | Total players: {server.unique_players}
                                    </div>
                                    <div className="flex justify-between items-center text-sm">
                                        <span className="bg-gray-700 px-2 py-0.5 rounded text-white">
                                            Online: {server.online}/{server.max}
                                        </span>
                                        <span className="text-xs text-gray-500">
                                            {formatDistanceToNow(new Date(server.updated), { addSuffix: true, locale: enUS })}
                                        </span>
                                    </div>
                                </Link>
                            );
                        })
                    ))}
                    
                    {data?.pages[0]?.length === 0 && (
                        <div className="col-span-full text-center text-gray-500">Server list is empty</div>
                    )}
                </div>
            )}

            {isFetchingNextPage && (
                <div className="text-center py-4 text-gray-400">Loading...</div>
            )}

            {!hasNextPage && !isLoading && (data?.pages?.[0]?.length ?? 0) > 0 && (
                <div className="mt-8 p-4 text-center border-t border-gray-800 text-gray-500 italic">
                    🎉 This is the end
                </div>
            )}
        </div>
    );
};
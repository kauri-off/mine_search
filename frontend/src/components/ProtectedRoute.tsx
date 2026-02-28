import { useQuery } from "@tanstack/react-query";
import { serverApi } from "@/api/client";
import { Spinner } from "@/components";

interface ProtectedRouteProps {
  children: React.ReactNode;
}

export const ProtectedRoute = ({ children }: ProtectedRouteProps) => {
  const { isLoading, isError } = useQuery({
    queryKey: ["stats"],
    queryFn: serverApi.fetchStats,
    retry: false,
  });

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Spinner className="w-10 h-10" />
      </div>
    );
  }

  // On 401, the API interceptor already redirects to /login
  if (isError) return null;

  return <>{children}</>;
};

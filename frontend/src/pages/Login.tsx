import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Lock } from "lucide-react";
import axios from "axios";
import { authApi } from "@/api/client";
import { useTranslation } from "@/i18n";

export const Login = () => {
  const { t } = useTranslation();
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isPending, setIsPending] = useState(false);
  const navigate = useNavigate();

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsPending(true);
    try {
      await authApi.login({ password });
      navigate("/");
    } catch (err) {
      if (axios.isAxiosError(err)) {
        if (err.response?.status === 401) {
          setError(t.login.wrongPassword);
        } else if (err.response?.status === 429) {
          setError(t.login.tooManyRequests);
        } else {
          setError(t.login.networkError);
        }
      } else {
        setError(t.login.networkError);
      }
    } finally {
      setIsPending(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-[#0a0a0f]">
      <div className="w-full max-w-sm">
        {/* Logo */}
        <div className="text-center mb-8">
          <span className="text-2xl font-bold text-slate-100 tracking-tight">
            Mine<span className="text-indigo-400">Search</span>
          </span>
        </div>

        <form
          onSubmit={handleLogin}
          className="bg-[#111118] border border-[#2a2a3a] rounded-2xl p-8 shadow-2xl shadow-black/40"
        >
          <h2 className="text-base font-semibold text-slate-200 mb-6 text-center">
            {t.login.title}
          </h2>

          {error && (
            <div className="mb-4 px-3 py-2 bg-red-950/40 border border-red-800/40 text-red-400 text-sm rounded-lg text-center">
              {error}
            </div>
          )}

          <div className="mb-5">
            <label className="block mb-1.5 text-xs font-medium text-slate-500">
              {t.login.token}
            </label>
            <div className="relative">
              <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-600" />
              <input
                type="password"
                autoFocus
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full pl-10 pr-4 py-2.5 bg-[#1a1a24] border border-[#2a2a3a] rounded-lg text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
              />
            </div>
          </div>

          <button
            type="submit"
            disabled={isPending || !password}
            className="w-full py-2.5 rounded-lg text-sm font-semibold bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isPending ? "..." : t.login.login}
          </button>
        </form>
      </div>
    </div>
  );
};

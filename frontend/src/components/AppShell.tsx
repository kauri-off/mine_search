import { useState } from "react";
import { NavLink } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { LayoutDashboard, BarChart3, Users, Cpu, Menu, X, RefreshCw } from "lucide-react";
import { cn } from "@/cn";
import { systemApi } from "@/api/client";
import { LanguageSwitcher } from "./LanguageSwitcher";
import { useTranslation } from "@/i18n";

interface AppShellProps {
  children: React.ReactNode;
}

export const AppShell = ({ children }: AppShellProps) => {
  const { t } = useTranslation();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  // Triggers a watchtower run (pull new images + recreate containers). Shared by
  // the desktop and mobile sidebars, so both reflect the in-flight state.
  const update = useMutation({ mutationFn: systemApi.triggerUpdate });

  const navLinkClass = ({ isActive }: { isActive: boolean }) =>
    cn(
      "flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
      isActive
        ? "bg-indigo-600/20 text-indigo-300 border border-indigo-600/30"
        : "text-slate-400 hover:text-slate-200 hover:bg-white/5",
    );

  const sidebarContent = (onNav?: () => void) => (
    <>
      {/* Logo */}
      <div className="px-5 py-5 border-b border-[#2a2a3a] flex items-center justify-between">
        <span className="text-lg font-bold text-slate-100 tracking-tight">
          Mine<span className="text-indigo-400">Search</span>
        </span>
        {onNav && (
          <button
            onClick={onNav}
            className="p-1 text-slate-400 hover:text-slate-200 md:hidden"
          >
            <X className="w-5 h-5" />
          </button>
        )}
      </div>

      {/* Nav */}
      <nav className="flex-1 px-3 py-4 space-y-1">
        <NavLink to="/" end className={navLinkClass} onClick={onNav}>
          <LayoutDashboard className="w-4 h-4 flex-shrink-0" />
          {t.dashboard.title}
        </NavLink>
        <NavLink to="/stats" className={navLinkClass} onClick={onNav}>
          <BarChart3 className="w-4 h-4 flex-shrink-0" />
          {t.stats.title}
        </NavLink>
        <NavLink to="/players" className={navLinkClass} onClick={onNav}>
          <Users className="w-4 h-4 flex-shrink-0" />
          {t.players.title}
        </NavLink>
        <NavLink to="/workers" className={navLinkClass} onClick={onNav}>
          <Cpu className="w-4 h-4 flex-shrink-0" />
          {t.workers.title}
        </NavLink>
      </nav>

      {/* Footer */}
      <div className="px-3 py-4 border-t border-[#2a2a3a] space-y-2">
        <button
          onClick={() => {
            if (window.confirm(t.system.updateConfirm)) update.mutate();
          }}
          disabled={update.isPending}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium bg-indigo-600/20 border border-indigo-600/40 text-indigo-300 hover:bg-indigo-600/30 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <RefreshCw className={cn("w-4 h-4 flex-shrink-0", update.isPending && "animate-spin")} />
          {update.isPending ? t.system.updating : t.system.updateStack}
        </button>
        {update.isSuccess && (
          <p className="text-xs text-green-400 text-center">{t.system.updateStarted}</p>
        )}
        {update.isError && (
          <p className="text-xs text-red-400 text-center">{t.system.updateError}</p>
        )}
        <LanguageSwitcher />
      </div>
    </>
  );

  return (
    <div className="flex h-screen flex-col md:flex-row">
      {/* Mobile header */}
      <header className="flex md:hidden items-center px-4 h-12 bg-[#111118] border-b border-[#2a2a3a] flex-shrink-0">
        <button
          onClick={() => setSidebarOpen(true)}
          className="p-1 text-slate-400 hover:text-slate-200 mr-3"
        >
          <Menu className="w-5 h-5" />
        </button>
        <span className="text-base font-bold text-slate-100 tracking-tight">
          Mine<span className="text-indigo-400">Search</span>
        </span>
      </header>

      {/* Desktop sidebar */}
      <aside className="hidden md:flex w-56 flex-shrink-0 bg-[#111118] border-r border-[#2a2a3a] flex-col">
        {sidebarContent()}
      </aside>

      {/* Mobile sidebar overlay */}
      <div
        className={`fixed inset-0 z-40 bg-black/60 md:hidden transition-opacity duration-200 ${sidebarOpen ? "opacity-100 pointer-events-auto" : "opacity-0 pointer-events-none"}`}
        onClick={() => setSidebarOpen(false)}
      />
      <aside className={`fixed inset-y-0 left-0 z-50 w-56 bg-[#111118] border-r border-[#2a2a3a] flex flex-col md:hidden transition-transform duration-200 ease-in-out ${sidebarOpen ? "translate-x-0" : "-translate-x-full"}`}>
        {sidebarContent(() => setSidebarOpen(false))}
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-hidden flex flex-col min-w-0">
        {children}
      </main>
    </div>
  );
};

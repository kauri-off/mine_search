import { useState } from "react";
import { NavLink } from "react-router-dom";
import { LayoutDashboard, BarChart3, Menu, X } from "lucide-react";
import { cn } from "@/cn";
import { LanguageSwitcher } from "./LanguageSwitcher";
import { useTranslation } from "@/i18n";

interface AppShellProps {
  children: React.ReactNode;
}

export const AppShell = ({ children }: AppShellProps) => {
  const { t } = useTranslation();
  const [sidebarOpen, setSidebarOpen] = useState(false);

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
      </nav>

      {/* Footer */}
      <div className="px-3 py-4 border-t border-[#2a2a3a]">
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
      {sidebarOpen && (
        <>
          <div
            className="fixed inset-0 z-40 bg-black/60 md:hidden"
            onClick={() => setSidebarOpen(false)}
          />
          <aside className="fixed inset-y-0 left-0 z-50 w-56 bg-[#111118] border-r border-[#2a2a3a] flex flex-col md:hidden">
            {sidebarContent(() => setSidebarOpen(false))}
          </aside>
        </>
      )}

      {/* Main content */}
      <main className="flex-1 overflow-hidden flex flex-col min-w-0">
        {children}
      </main>
    </div>
  );
};

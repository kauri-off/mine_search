import { cn } from "@/cn";
import { useState } from "react";
import { Copy, Check } from "lucide-react";

// ---------------------------------------------------------------------------
// CopyButton
// ---------------------------------------------------------------------------

interface CopyButtonProps {
  text: string;
}

export const CopyButton = ({ text }: CopyButtonProps) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      title={copied ? "Copied!" : "Copy"}
      aria-label={copied ? "Copied!" : "Copy"}
      className={cn(
        "relative inline-flex items-center justify-center w-6 h-6 rounded-md transition-all duration-150",
        copied
          ? "bg-green-900/40 text-green-400"
          : "bg-[#1a1a24] hover:bg-[#2a2a3a] text-slate-500 hover:text-slate-300",
      )}
    >
      {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}

      <span
        className={cn(
          "absolute -top-7 left-1/2 -translate-x-1/2",
          "text-xs px-2 py-0.5 rounded bg-[#111118] border border-[#2a2a3a]",
          "whitespace-nowrap pointer-events-none transition-opacity duration-150",
          copied ? "opacity-100" : "opacity-0",
        )}
      >
        Copied!
      </span>
    </button>
  );
};

// ---------------------------------------------------------------------------
// ToggleButton
// ---------------------------------------------------------------------------

interface ToggleButtonProps {
  label: string;
  active: boolean;
  onClick: () => void;
  color?: "blue" | "red";
}

export const ToggleButton = ({
  label,
  active,
  onClick,
  color = "blue",
}: ToggleButtonProps) => (
  <button
    onClick={onClick}
    aria-pressed={active}
    aria-label={label}
    className={cn(
      "w-full py-2 px-3 rounded-lg text-sm font-medium transition-colors flex justify-between items-center",
      active
        ? color === "red"
          ? "bg-red-600/20 border border-red-600/40 text-red-300"
          : "bg-indigo-600/20 border border-indigo-600/40 text-indigo-300"
        : "bg-[#1a1a24] border border-[#2a2a3a] text-slate-500 hover:text-slate-300 hover:border-[#3a3a4a]",
    )}
  >
    <span>{label}</span>
    <span
      className={cn(
        "text-xs px-1.5 py-0.5 rounded font-medium",
        active ? "bg-black/20" : "bg-black/20 text-slate-600",
      )}
    >
      {active ? "ON" : "OFF"}
    </span>
  </button>
);

// ---------------------------------------------------------------------------
// StatusBlock
// ---------------------------------------------------------------------------

const STATUS_COLORS: Record<string, { active: string; inactive: string }> = {
  gray: {
    active: "bg-slate-600/30 text-slate-300 border-slate-500/50",
    inactive: "bg-[#1a1a24] text-slate-600 border-[#2a2a3a]",
  },
  blue: {
    active: "bg-indigo-600/20 text-indigo-300 border-indigo-500/40",
    inactive: "bg-[#1a1a24] text-slate-600 border-[#2a2a3a]",
  },
  amber: {
    active: "bg-amber-500/20 text-amber-300 border-amber-500/40",
    inactive: "bg-[#1a1a24] text-slate-600 border-[#2a2a3a]",
  },
};

interface StatusBlockProps {
  label: string;
  active: boolean;
  activeColor: string;
  onClick?: () => void;
}

export const StatusBlock = ({
  label,
  active,
  activeColor,
  onClick,
}: StatusBlockProps) => {
  const colors = STATUS_COLORS[activeColor] ?? STATUS_COLORS.gray;

  return (
    <span
      onClick={onClick}
      role={onClick ? "button" : undefined}
      aria-label={label}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => e.key === "Enter" && onClick() : undefined}
      className={cn(
        "px-2 py-0.5 rounded border text-xs font-medium transition-colors select-none",
        active
          ? `${colors.active} cursor-default`
          : `${colors.inactive} cursor-pointer hover:border-slate-500 hover:text-slate-400`,
      )}
    >
      {label}
    </span>
  );
};

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

interface SpinnerProps {
  className?: string;
}

export const Spinner = ({ className }: SpinnerProps) => (
  <svg
    className={cn("animate-spin w-8 h-8 text-indigo-500", className)}
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    aria-label="Loading"
    role="status"
  >
    <circle
      className="opacity-25"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      strokeWidth="4"
    />
    <path
      className="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
    />
  </svg>
);

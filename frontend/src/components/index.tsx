import { cn } from "@/lib/cn";
import { useState } from "react";

// ---------------------------------------------------------------------------
// CopyButton
// ---------------------------------------------------------------------------

interface CopyButtonProps {
  text: string;
}

export const CopyButton = ({ text }: CopyButtonProps) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // Fallback for older browsers
      const el = document.createElement("textarea");
      el.value = text;
      document.body.appendChild(el);
      el.select();
      document.execCommand("copy");
      document.body.removeChild(el);
    }

    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      title={copied ? "Copied!" : "Copy IP"}
      className={cn(
        "relative inline-flex items-center justify-center w-7 h-7 rounded-md transition-all duration-200",
        copied
          ? "bg-green-500/20 text-green-400 scale-95"
          : "bg-gray-700 hover:bg-gray-600 text-gray-400 hover:text-white",
      )}
    >
      {copied ? (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2.5}
          strokeLinecap="round"
          strokeLinejoin="round"
          style={{ animation: "pop 0.2s ease-out" }}
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
      ) : (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <rect x="9" y="2" width="6" height="4" rx="1" ry="1" />
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
        </svg>
      )}

      <span
        className={cn(
          "absolute -top-8 left-1/2 -translate-x-1/2",
          "text-xs px-2 py-1 rounded bg-gray-900 border border-gray-700",
          "whitespace-nowrap pointer-events-none transition-opacity duration-150",
          copied ? "opacity-100" : "opacity-0",
        )}
      >
        Copied!
      </span>

      <style>{`
        @keyframes pop {
          0%   { transform: scale(0.6); opacity: 0.5; }
          60%  { transform: scale(1.2); }
          100% { transform: scale(1);   opacity: 1; }
        }
      `}</style>
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
    className={cn(
      "w-full py-2 px-4 rounded font-medium transition flex justify-between items-center",
      active
        ? color === "red"
          ? "bg-red-600 hover:bg-red-700"
          : "bg-blue-600 hover:bg-blue-700"
        : "bg-gray-700 hover:bg-gray-600",
    )}
  >
    <span>{label}</span>
    <span className="text-xs uppercase bg-black/20 px-2 py-0.5 rounded">
      {active ? "ON" : "OFF"}
    </span>
  </button>
);

// ---------------------------------------------------------------------------
// StatusBlock
// ---------------------------------------------------------------------------

const STATUS_COLORS: Record<string, { active: string; inactive: string }> = {
  gray: {
    active: "bg-gray-500/30 text-gray-300 border-gray-500",
    inactive: "bg-gray-800 text-gray-600 border-gray-700",
  },
  blue: {
    active: "bg-blue-500/20 text-blue-300 border-blue-500",
    inactive: "bg-gray-800 text-gray-600 border-gray-700",
  },
  amber: {
    active: "bg-amber-500/20 text-amber-300 border-amber-500",
    inactive: "bg-gray-800 text-gray-600 border-gray-700",
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
      className={cn(
        "px-2 py-0.5 rounded border text-xs font-medium transition-colors select-none",
        active
          ? `${colors.active} cursor-default`
          : `${colors.inactive} cursor-pointer hover:border-gray-500 hover:text-gray-400`,
      )}
    >
      {label}
    </span>
  );
};

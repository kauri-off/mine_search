import { useTranslation } from "@/i18n";
import type { Language } from "@/i18n";
import { useState, useRef, useEffect } from "react";
import ReactCountryFlag from "react-country-flag";
import { cn } from "@/cn";

const LANGUAGES: { code: Language; label: string; countryCode: string }[] = [
  { code: "en", label: "English", countryCode: "GB" },
  { code: "ru", label: "Русский", countryCode: "RU" },
];

export const LanguageSwitcher = () => {
  const { lang, setLang } = useTranslation();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const current = LANGUAGES.find((l) => l.code === lang) ?? LANGUAGES[0];

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 w-full px-3 py-2 rounded-lg bg-transparent hover:bg-white/5 text-slate-400 hover:text-slate-200 transition-colors text-sm"
      >
        <ReactCountryFlag
          countryCode={current.countryCode}
          svg
          style={{ width: "1rem", height: "auto" }}
        />
        <span>{current.label}</span>
        <svg
          className={cn("w-3 h-3 ml-auto transition-transform", open && "rotate-180")}
          viewBox="0 0 10 6"
          fill="none"
        >
          <path
            d="M1 1l4 4 4-4"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>

      {open && (
        <div className="absolute bottom-full left-0 mb-1 w-full rounded-lg bg-[#1a1a24] border border-[#2a2a3a] shadow-xl z-50">
          {LANGUAGES.map(({ code, label, countryCode }) => (
            <button
              key={code}
              onClick={() => {
                setLang(code);
                setOpen(false);
              }}
              className={cn(
                "flex items-center gap-2 w-full px-3 py-2 text-sm rounded-lg hover:bg-white/5 transition-colors",
                lang === code ? "text-indigo-300" : "text-slate-400",
              )}
            >
              <ReactCountryFlag
                countryCode={countryCode}
                svg
                style={{ width: "1rem", height: "auto" }}
              />
              <span>{label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

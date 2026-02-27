import { useTranslation } from "@/i18n";
import type { Language } from "@/i18n";
import { useState, useRef, useEffect } from "react";
import ReactCountryFlag from "react-country-flag";

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
        className="flex items-center gap-2 px-3 py-1.5 rounded bg-gray-700 text-gray-200 hover:bg-gray-600 transition-colors text-sm font-medium"
      >
        <ReactCountryFlag countryCode={current.countryCode} svg style={{ width: "1.25rem", height: "auto" }} />
        <span>{current.label}</span>
        <svg
          className={`w-3 h-3 ml-1 transition-transform ${open ? "rotate-180" : ""}`}
          viewBox="0 0 10 6"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path d="M1 1l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </button>

      {open && (
        <div className="absolute right-0 mt-1 w-40 rounded bg-gray-800 border border-gray-700 shadow-lg z-50">
          {LANGUAGES.map(({ code, label, countryCode }) => (
            <button
              key={code}
              onClick={() => { setLang(code); setOpen(false); }}
              className={`flex items-center gap-2 w-full px-3 py-2 text-sm hover:bg-gray-700 transition-colors ${
                lang === code ? "text-blue-400 font-semibold" : "text-gray-200"
              }`}
            >
              <ReactCountryFlag countryCode={countryCode} svg style={{ width: "1.25rem", height: "auto" }} />
              <span>{label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

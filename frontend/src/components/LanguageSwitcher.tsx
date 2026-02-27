import { useTranslation } from "@/i18n";
import type { Language } from "@/i18n";

const LANGUAGES: { code: Language; label: string }[] = [
  { code: "en", label: "EN" },
  { code: "ru", label: "RU" },
];

export const LanguageSwitcher = () => {
  const { lang, setLang } = useTranslation();

  return (
    <div className="flex gap-1">
      {LANGUAGES.map(({ code, label }) => (
        <button
          key={code}
          onClick={() => setLang(code)}
          className={`px-2 py-1 rounded text-sm font-medium transition-colors ${
            lang === code
              ? "bg-blue-600 text-white"
              : "bg-gray-700 text-gray-300 hover:bg-gray-600"
          }`}
        >
          {label}
        </button>
      ))}
    </div>
  );
};

import {
  createContext,
  useContext,
  useState,
  type ReactNode,
} from "react";
import type { Translations } from "./translations";
import { en } from "./en";
import { ru } from "./ru";

export type Language = "en" | "ru";

const STORAGE_KEY = "lang";

const TRANSLATIONS: Record<Language, Translations> = { en, ru };

function detectLanguage(): Language {
  const saved = localStorage.getItem(STORAGE_KEY) as Language | null;
  if (saved && saved in TRANSLATIONS) return saved;

  const browserLang = navigator.language.split("-")[0];
  return browserLang in TRANSLATIONS ? (browserLang as Language) : "en";
}

interface LanguageContextValue {
  lang: Language;
  setLang: (lang: Language) => void;
  t: Translations;
}

const LanguageContext = createContext<LanguageContextValue | null>(null);

export const LanguageProvider = ({ children }: { children: ReactNode }) => {
  const [lang, setLangState] = useState<Language>(detectLanguage);

  const setLang = (newLang: Language) => {
    localStorage.setItem(STORAGE_KEY, newLang);
    setLangState(newLang);
  };

  return (
    <LanguageContext.Provider value={{ lang, setLang, t: TRANSLATIONS[lang] }}>
      {children}
    </LanguageContext.Provider>
  );
};

export const useTranslation = () => {
  const ctx = useContext(LanguageContext);
  if (!ctx)
    throw new Error("useTranslation must be used within LanguageProvider");
  return ctx;
};

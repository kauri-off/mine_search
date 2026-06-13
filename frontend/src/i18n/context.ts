import { createContext, useContext } from "react";
import type { Translations } from "./translations";
import { en } from "./en";
import { ru } from "./ru";

export type Language = "en" | "ru";

const STORAGE_KEY = "lang";

export const TRANSLATIONS: Record<Language, Translations> = { en, ru };

export function detectLanguage(): Language {
  const saved = localStorage.getItem(STORAGE_KEY) as Language | null;
  if (saved && saved in TRANSLATIONS) return saved;

  const browserLang = navigator.language.split("-")[0];
  return browserLang in TRANSLATIONS ? (browserLang as Language) : "en";
}

export function storeLanguage(lang: Language) {
  localStorage.setItem(STORAGE_KEY, lang);
}

export interface LanguageContextValue {
  lang: Language;
  setLang: (lang: Language) => void;
  t: Translations;
}

export const LanguageContext = createContext<LanguageContextValue | null>(null);

export const useTranslation = () => {
  const ctx = useContext(LanguageContext);
  if (!ctx)
    throw new Error("useTranslation must be used within LanguageProvider");
  return ctx;
};

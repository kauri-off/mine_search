import { useState, type ReactNode } from "react";
import {
  LanguageContext,
  TRANSLATIONS,
  detectLanguage,
  storeLanguage,
  type Language,
} from "./context";

export const LanguageProvider = ({ children }: { children: ReactNode }) => {
  const [lang, setLangState] = useState<Language>(detectLanguage);

  const setLang = (newLang: Language) => {
    storeLanguage(newLang);
    setLangState(newLang);
  };

  return (
    <LanguageContext.Provider value={{ lang, setLang, t: TRANSLATIONS[lang] }}>
      {children}
    </LanguageContext.Provider>
  );
};

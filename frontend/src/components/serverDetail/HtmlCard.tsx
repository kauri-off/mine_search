import DOMPurify from "dompurify";

interface HtmlCardProps {
  title: string;
  html: string;
}

export const HtmlCard = ({ title, html }: HtmlCardProps) => (
  <div className="bg-panel border border-border rounded-xl p-5 overflow-hidden">
    <h3 className="text-sm font-semibold text-slate-300 mb-3">{title}</h3>
    <div
      className="prose prose-invert prose-xs max-w-none bg-app-elevated px-3 py-2 rounded-lg font-mono text-xs leading-relaxed"
      dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(html) }}
    />
  </div>
);

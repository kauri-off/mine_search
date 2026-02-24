interface HtmlCardProps {
  title: string;
  html: string;
}

export const HtmlCard = ({ title, html }: HtmlCardProps) => (
  <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 overflow-hidden">
    <h3 className="font-bold mb-4">{title}</h3>
    <div
      className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  </div>
);

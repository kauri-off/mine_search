interface InfoRowProps {
  label: string;
  children: React.ReactNode;
}

export const InfoRow = ({ label, children }: InfoRowProps) => (
  <div className="flex justify-between border-b border-gray-700 pb-2">
    <span>{label}:</span>
    <span>{children}</span>
  </div>
);

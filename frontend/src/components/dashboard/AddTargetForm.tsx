import { useTranslation } from "@/i18n";

const IP_PORT_REGEX = /^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?$/;

const isValidIp = (value: string) => IP_PORT_REGEX.test(value.trim());

interface AddTargetFormProps {
  ip: string;
  isPending: boolean;
  error?: string | null;
  onChange: (value: string) => void;
  onSubmit: () => void;
}

export const AddTargetForm = ({
  ip,
  isPending,
  error,
  onChange,
  onSubmit,
}: AddTargetFormProps) => {
  const { t } = useTranslation();
  const showFormatError = ip.trim() !== "" && !isValidIp(ip);

  return (
    <div className="mb-6 p-4 bg-gray-800 rounded-lg">
      <div className="flex gap-3 items-center">
        <span className="hidden sm:inline text-gray-400 flex-shrink-0">
          {t.addIp.label}
        </span>
        <input
          type="text"
          value={ip}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && !showFormatError && onSubmit()}
          placeholder="e.g. 192.168.1.1"
          aria-label="Server IP address"
          className="min-w-0 flex-1 bg-gray-700 text-white placeholder-gray-500 rounded-md px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          onClick={onSubmit}
          disabled={isPending || !ip.trim() || showFormatError}
          aria-label={t.addIp.add}
          className="flex-shrink-0 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm font-medium px-4 py-2 rounded-md transition-colors"
        >
          {isPending ? t.addIp.adding : t.addIp.add}
        </button>
      </div>
      {showFormatError && (
        <p className="mt-2 text-xs text-yellow-400">{t.addIp.invalidFormat}</p>
      )}
      {error && <p className="mt-2 text-xs text-red-400">{error}</p>}
    </div>
  );
};

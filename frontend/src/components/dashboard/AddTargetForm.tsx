import { useState } from "react";
import { Plus } from "lucide-react";
import { Modal } from "@/components/Modal";
import { useTranslation } from "@/i18n";

const IP_REGEX = /^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?$/;

interface AddTargetFormProps {
  isOpen: boolean;
  onClose: () => void;
  isPending: boolean;
  error: string | null;
  onSubmit: (addr: string) => void;
}

export const AddTargetForm = ({
  isOpen,
  onClose,
  isPending,
  error,
  onSubmit,
}: AddTargetFormProps) => {
  const { t } = useTranslation();
  const [ip, setIp] = useState("");
  const [formatError, setFormatError] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = ip.trim();
    if (!IP_REGEX.test(trimmed)) {
      setFormatError(true);
      return;
    }
    setFormatError(false);
    onSubmit(trimmed);
  };

  const handleClose = () => {
    setIp("");
    setFormatError(false);
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title={t.addIp.label}>
      <form onSubmit={handleSubmit} className="space-y-4">
        <input
          type="text"
          autoFocus
          value={ip}
          onChange={(e) => {
            setIp(e.target.value);
            setFormatError(false);
          }}
          placeholder="192.168.1.1 or 192.168.1.1:25565"
          className="w-full bg-[#1a1a24] border border-[#2a2a3a] rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
        />

        {formatError && (
          <p className="text-xs text-amber-400">{t.addIp.invalidFormat}</p>
        )}
        {error && <p className="text-xs text-red-400">{error}</p>}

        <button
          type="submit"
          disabled={isPending || !ip.trim()}
          className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <Plus className="w-4 h-4" />
          {isPending ? t.addIp.adding : t.addIp.add}
        </button>
      </form>
    </Modal>
  );
};

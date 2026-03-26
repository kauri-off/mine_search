import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { serverApi } from "@/api/client";
import { Modal } from "@/components/Modal";
import { useTranslation } from "@/i18n";

const IP_REGEX = /^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?$/;

function parseLines(raw: string): { valid: string[]; invalidCount: number } {
  const lines = raw.split("\n").map((l) => l.trim()).filter(Boolean);
  const valid: string[] = [];
  let invalidCount = 0;
  for (const line of lines) {
    if (IP_REGEX.test(line)) {
      valid.push(line);
    } else {
      invalidCount++;
    }
  }
  return { valid, invalidCount };
}

interface BulkImportModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const BulkImportModal = ({ isOpen, onClose }: BulkImportModalProps) => {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const [parsed, setParsed] = useState<{ valid: string[]; invalidCount: number } | null>(null);
  const [successCount, setSuccessCount] = useState<number | null>(null);

  const importMutation = useMutation({
    mutationFn: (addrs: string[]) =>
      serverApi.addTargetList(addrs.map((addr) => ({ addr, quick: false }))),
    onSuccess: () => {
      setSuccessCount(parsed?.valid.length ?? 0);
      setText("");
      setParsed(null);
    },
  });

  const handleClose = () => {
    setText("");
    setParsed(null);
    setSuccessCount(null);
    importMutation.reset();
    onClose();
  };

  const handleParse = () => {
    setParsed(parseLines(text));
    setSuccessCount(null);
    importMutation.reset();
  };

  const handleImport = () => {
    if (!parsed || parsed.valid.length === 0) return;
    importMutation.mutate(parsed.valid);
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title={t.bulkImport.title}>
      <div className="space-y-4">
        <textarea
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            setParsed(null);
            setSuccessCount(null);
          }}
          placeholder={t.bulkImport.placeholder}
          rows={6}
          className="w-full bg-[#1a1a24] border border-[#2a2a3a] rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 font-mono focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 resize-none transition-colors"
        />

        {parsed && (
          <div className="text-sm space-y-1">
            <p className="text-indigo-300">
              {parsed.valid.length} valid address{parsed.valid.length !== 1 ? "es" : ""}
            </p>
            {parsed.invalidCount > 0 && (
              <p className="text-amber-400">
                {t.bulkImport.invalidLines(parsed.invalidCount)}
              </p>
            )}
          </div>
        )}

        {successCount !== null && (
          <p className="text-sm text-green-400">{t.bulkImport.success(successCount)}</p>
        )}

        {importMutation.isError && (
          <p className="text-sm text-red-400">{t.bulkImport.error}</p>
        )}

        <div className="flex gap-3 pt-1">
          <button
            onClick={handleParse}
            disabled={!text.trim()}
            className="px-4 py-2 rounded-lg text-sm font-medium bg-[#1a1a24] border border-[#2a2a3a] text-slate-300 hover:text-slate-100 hover:border-[#3a3a4a] disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            {t.bulkImport.parse}
          </button>

          {parsed && parsed.valid.length > 0 && (
            <button
              onClick={handleImport}
              disabled={importMutation.isPending}
              className="flex-1 px-4 py-2 rounded-lg text-sm font-medium bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {importMutation.isPending
                ? t.bulkImport.importing
                : t.bulkImport.importN(parsed.valid.length)}
            </button>
          )}
        </div>
      </div>
    </Modal>
  );
};

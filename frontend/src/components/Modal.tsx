import { Dialog, DialogPanel, DialogTitle } from "@headlessui/react";
import { X } from "lucide-react";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

/**
 * Accessible modal built on Headless UI's `Dialog`, which provides a focus
 * trap, focus restoration on close, Escape-to-close, click-outside-to-close,
 * `role="dialog"`/`aria-modal`, and `aria-labelledby` wired to `DialogTitle` —
 * none of which the previous hand-rolled portal had. Rendered in a portal by
 * Headless UI; nothing renders while closed.
 */
export const Modal = ({ isOpen, onClose, title, children }: ModalProps) => {
  return (
    <Dialog open={isOpen} onClose={onClose} className="relative z-50">
      <div
        className="fixed inset-0 bg-black/60 backdrop-blur-sm transition duration-200 data-[closed]:opacity-0"
        aria-hidden="true"
      />
      <div className="fixed inset-0 flex items-center justify-center p-4">
        <DialogPanel
          transition
          className="relative w-full max-w-lg bg-panel border border-border rounded-2xl shadow-2xl shadow-black/60 flex flex-col max-h-[90vh] transition duration-200 data-[closed]:opacity-0 data-[closed]:scale-95"
        >
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-border">
            <DialogTitle className="text-base font-semibold text-slate-100">
              {title}
            </DialogTitle>
            <button
              onClick={onClose}
              className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-white/5 transition-colors"
              aria-label="Close"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
          {/* Body */}
          <div className="flex-1 overflow-y-auto px-6 py-4">{children}</div>
        </DialogPanel>
      </div>
    </Dialog>
  );
};

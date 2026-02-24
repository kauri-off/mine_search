interface AddIpFormProps {
  ip: string;
  isPending: boolean;
  onChange: (value: string) => void;
  onSubmit: () => void;
}

export const AddIpForm = ({
  ip,
  isPending,
  onChange,
  onSubmit,
}: AddIpFormProps) => (
  <div className="mb-6 p-4 bg-gray-800 rounded-lg flex gap-3 items-center">
    <span className="text-gray-400">Add IP:</span>
    <input
      type="text"
      value={ip}
      onChange={(e) => onChange(e.target.value)}
      onKeyDown={(e) => e.key === "Enter" && onSubmit()}
      placeholder="e.g. 192.168.1.1"
      className="flex-1 bg-gray-700 text-white placeholder-gray-500 rounded-md px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500"
    />
    <button
      onClick={onSubmit}
      disabled={isPending || !ip.trim()}
      className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm font-medium px-4 py-2 rounded-md transition-colors"
    >
      {isPending ? "Adding..." : "Add"}
    </button>
  </div>
);

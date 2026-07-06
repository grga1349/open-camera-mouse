import type { FC } from "react";

type ErrorBannerProps = {
  message: string;
  onDismiss: () => void;
};

export const ErrorBanner: FC<ErrorBannerProps> = ({ message, onDismiss }) => (
  <div className="flex items-center justify-between gap-3 rounded-2xl border border-red-800 bg-red-950 px-4 py-3 text-sm text-red-200">
    <span className="text-left">{message}</span>
    <button
      onClick={onDismiss}
      className="shrink-0 font-semibold uppercase tracking-wide text-red-300 hover:text-red-100"
    >
      Dismiss
    </button>
  </div>
);

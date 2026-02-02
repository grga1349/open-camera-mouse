import type { ButtonHTMLAttributes, FC } from "react";
import { cn } from "../lib/cn";

const baseClasses =
  "inline-flex items-center justify-center rounded-full border px-5 py-2 text-sm font-semibold transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:opacity-50";

const variantClasses = {
  action: "border-emerald-500 bg-emerald-500 text-white hover:bg-emerald-400 focus-visible:outline-emerald-400",
  highlight: "border-zinc-800 bg-zinc-900 text-zinc-100 hover:bg-zinc-800 focus-visible:outline-zinc-200",
  ghost: "border-transparent text-zinc-200 hover:bg-zinc-900 focus-visible:outline-zinc-200",
} as const;

type Variant = keyof typeof variantClasses;

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: Variant;
  fullWidth?: boolean;
};

export const Button: FC<ButtonProps> = ({ variant = "highlight", fullWidth, className, ...props }) => (
  <button
    className={cn(baseClasses, variantClasses[variant], fullWidth && "w-full", className)}
    {...props}
  />
);

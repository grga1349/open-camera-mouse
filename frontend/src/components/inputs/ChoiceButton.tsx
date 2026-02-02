import type { ButtonHTMLAttributes, FC, ReactNode } from "react";
import { Button } from "../Button";

type ChoiceButtonProps = {
  selected: boolean;
  children: ReactNode;
} & ButtonHTMLAttributes<HTMLButtonElement>;

export const ChoiceButton: FC<ChoiceButtonProps> = ({ selected, children, className = "", ...props }) => (
  <Button
    type="button"
    variant={selected ? "action" : "highlight"}
    className={`flex-1 text-sm ${selected ? "" : "border-zinc-800 bg-zinc-950 text-zinc-400 hover:bg-zinc-900"} ${className}`}
    {...props}
  >
    {children}
  </Button>
);

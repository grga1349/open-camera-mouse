import type {FC, ReactNode} from 'react';

type ScreenShellProps = {
    header?: ReactNode;
    footer?: ReactNode;
    children: ReactNode;
    className?: string;
    mainClassName?: string;
};

/**
 * Shared layout wrapper that keeps screens consistent.
 */
export const ScreenShell: FC<ScreenShellProps> = ({header, footer, children, className = '', mainClassName = ''}) => (
    <div className={`mx-auto flex h-screen max-w-sm flex-col gap-4 bg-zinc-950 px-5 py-4 text-zinc-100 ${className}`}>
        {header}
        <main className={`flex flex-1 flex-col ${mainClassName}`}>
            {children}
        </main>
        {footer}
    </div>
);


import { JSX, useState } from "react";

interface PveButtonProps {
    icon: JSX.Element;
    tooltip: string | null;
    disabled?: boolean;
}

export function PveButton({ icon, tooltip, disabled = false }: PveButtonProps) {

    const [tooltipOpen, setTooltipOpen] = useState<boolean>(false);

    const baseBtnCls = [
        "relative text-foreground-dark/90 bg-card-dark p-2 rounded-md border-[1px] border-primary-dark/20 transition-transform duration-150 ease-out",
        disabled ? "opacity-40 grayscale pointer-events-none cursor-not-allowed transform-none" : "hover:scale-105"
    ].join(" ");

    const tooltipBg = disabled ? "bg-primary-dark/30 text-foreground-dark/70" : "bg-primary-dark text-foreground-dark";
    const tooltipStateCls = tooltipOpen && !disabled ? "opacity-100 scale-100 pointer-events-auto" : "opacity-0 scale-95 pointer-events-none";

    return (
        <div className="relative flex flex-col items-center" aria-disabled={disabled}>
            <div
                className={baseBtnCls}
                onMouseEnter={() => !disabled && setTooltipOpen(true)}
                onMouseLeave={() => !disabled && setTooltipOpen(false)}
            >
                {icon}
            </div>
            {
                tooltip &&
                <div
                    className={`absolute z-20 -top-10 left-1/2 -translate-x-1/2 text-sm px-2 py-[2px] rounded shadow w-fit flex items-center transition-all duration-200 ease-out ${tooltipBg} ${tooltipStateCls}`}
                >
                    <span className="whitespace-nowrap">{tooltip}</span>
                    <div className={`absolute left-1/2 bottom-0 -translate-x-1/2 translate-y-1 w-2 h-2 rotate-45 z-10 ${disabled ? "bg-primary-dark/30" : "bg-primary-dark"}`}></div>
                </div>
            }
        </div>
    );
}
import { JSX, useState } from "react";

interface TopBarButtonProps {
    icon: JSX.Element;
    tooltip: string | null;
    disabled?: boolean;
    active?: boolean;
    onClick?: () => void; // Added onClick prop
}

export function TopBarButton({ icon, tooltip, disabled = false, active = false, onClick }: TopBarButtonProps) {
    const [tooltipOpen, setTooltipOpen] = useState<boolean>(false);

    const baseBtnCls = [
        "transition-colors rounded-full p-2 border",
        "border-transparent",
        "relative",
        "text-foreground-dark/90",
        active && !disabled ? "bg-primary/70" : "bg-card-dark",
        !disabled ? "hover:bg-primary/50 cursor-pointer" : "opacity-40 grayscale pointer-events-none cursor-not-allowed",
    ]
        .filter(Boolean)
        .join(" ");

    const tooltipBg = disabled ? "bg-primary-dark/30 text-foreground-dark/70" : "bg-primary-dark text-foreground-dark";
    const tooltipStateCls = tooltipOpen && !disabled ? "opacity-100 scale-100 pointer-events-auto" : "opacity-0 scale-95 pointer-events-none";

    return (
        <div className="relative flex flex-col items-center" aria-disabled={disabled}>
            <button
                type="button"
                className={baseBtnCls}
                onMouseEnter={() => !disabled && setTooltipOpen(true)}
                onMouseLeave={() => !disabled && setTooltipOpen(false)}
                onClick={!disabled && onClick ? onClick : undefined} // Added onClick handler
                disabled={disabled}
                tabIndex={disabled ? -1 : 0}
            >
                {icon}
            </button>
            {
                tooltip &&
                <div
                    className={`absolute z-20 top-full mt-2 left-1/2 -translate-x-1/2 text-sm px-2 py-[2px] rounded shadow w-fit flex items-center transition-all duration-200 ease-out ${tooltipBg} ${tooltipStateCls}`}
                >
                    <span className="whitespace-nowrap">{tooltip}</span>
                    <div className={`absolute left-1/2 top-0 -translate-x-1/2 -translate-y-1 w-2 h-2 rotate-45 z-10 ${disabled ? "bg-primary-dark/30" : "bg-primary-dark"}`}></div>
                </div>
            }
        </div>
    );
}
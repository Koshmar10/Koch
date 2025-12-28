import { ChessKing, Clock, User } from "lucide-react";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";

interface Props {
    display: boolean;
    color: PieceColor;
    player: string;
    rating?: string | number;
    clock?: string | number;      // new: clock display (e.g. "0:09:58")
    isTurn?: boolean;             // new: whether it's this player's turn
}

export function PlayerCard({ display, player, color, rating, clock, isTurn }: Props) {
    if (!display) return null;

    const containerClasses = `
        flex items-center gap-3 w-full px-3 py-2 rounded-lg bg-card-dark shadow-sm transition-colors
        border-[1px] border-border-dark/80
    `;

    return (
        <div className={containerClasses}>
            {/* Avatar Square */}
            <div className={`
                relative flex items-center justify-center w-10 h-10 rounded-md shadow-inner
                ${color === "White" ? "bg-gray-200 text-gray-900" : "bg-gray-900 text-gray-100"}
            `}>
                <ChessKing className="w-6 h-6 opacity-80" />

                {/* Small Color Indicator Badge */}
                <div className={`
                    absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-[#1a1a1a]
                    ${color === "White" ? "bg-white" : "bg-black border-white/60 border-[1px]"}
                `} />
            </div>

            {/* Player Details */}
            <div className="flex-1 flex items-center justify-between">
                <div className="flex flex-col justify-center min-w-0">
                    <span className="text-lg font-semibold text-foreground-dark/90 tracking-tight leading-tight truncate max-w-[300px]">
                        {player}
                    </span>

                </div>

                {/* Right side: clock and turn indicator */}
                <div className="flex flex-col items-end ml-3">
                    {typeof clock !== "undefined" && (
                        <div className={`flex items-center gap-2 ${isTurn ? "text-foreground-dark" : "text-foreground-dark/40"}`}>
                            <Clock className="w-3.5 h-3.5 text-foreground-dark/40" />
                            <span className="font-mono text-xl  tracking-widest">{clock.toString().slice(0, -3)}</span>
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}
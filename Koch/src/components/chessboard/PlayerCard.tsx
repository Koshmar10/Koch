import { ChessKing, User } from "lucide-react";
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
        flex items-center gap-3 w-full px-3 py-2 rounded-lg bg-primary/20 border border-white/5 shadow-sm transition-colors
        ${isTurn ? "ring-2 ring-accent/50 bg-primary/30" : "hover:bg-primary/30"}
    `;

    return (
        <div className={containerClasses}>
            {/* Avatar Square */}
            <div className={`
                relative flex items-center justify-center w-10 h-10 rounded-md shadow-inner
                ${color === "White" ? "bg-gray-200 text-gray-900" : "bg-gray-800 text-gray-100"}
            `}>
                <ChessKing className="w-6 h-6 opacity-80" />

                {/* Small Color Indicator Badge */}
                <div className={`
                    absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-[#1a1a1a]
                    ${color === "White" ? "bg-white" : "bg-black"}
                `} />
            </div>

            {/* Player Details */}
            <div className="flex-1 flex items-center justify-between">
                <div className="flex flex-col justify-center min-w-0">
                    <span className="text-sm font-bold text-secondary tracking-tight leading-tight truncate max-w-[150px]">
                        {player}
                    </span>
                    <div className="flex items-center gap-1.5 mt-0.5">
                        <span className="text-[10px] font-mono text-secondary/50 bg-white/5 px-1.5 py-0.5 rounded">
                            {rating ?? "Unrated"}
                        </span>
                    </div>
                </div>

                {/* Right side: clock and turn indicator */}
                <div className="flex flex-col items-end ml-3">
                    {typeof clock !== "undefined" && (
                        <div className={`
                            px-3 py-1 rounded-lg shadow bg-black/80 border border-white/10
                            flex items-center justify-center min-w-[70px]
                            ${isTurn ? "ring-2 ring-accent/60" : ""}
                        `}>
                            <span className={`text-md font-mono tracking-widest ${isTurn ? "text-accent font-bold" : "text-secondary/90"}`}>
                                {clock}
                            </span>
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}
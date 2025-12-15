import { ChessKing, User } from "lucide-react";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";

interface Props {
    display: boolean;
    color: PieceColor;
    player: string;
    rating?: string | number;
}

export function PlayerCard({ display, player, color, rating }: Props) {
    if (!display) return null;

    return (
        <div className="flex items-center gap-3 w-full px-3 py-2 rounded-lg bg-primary/20 border border-white/5 shadow-sm transition-colors hover:bg-primary/30">
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
            <div className="flex flex-col justify-center">
                <span className="text-sm font-bold text-secondary tracking-tight leading-tight truncate max-w-[150px]">
                    {player}
                </span>
                <div className="flex items-center gap-1.5 mt-0.5">
                    <span className="text-[10px] font-mono text-secondary/50 bg-white/5 px-1.5 py-0.5 rounded">
                        {rating || "Unrated"}
                    </span>
                </div>
            </div>
        </div>
    )
}
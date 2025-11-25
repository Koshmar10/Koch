import { useState } from "react"
import { removeDefaultForButton } from "../App";
import { ChessPawn, ChessKnight, ChessBishop, ChessRook, ChessQueen, ChessKing } from "lucide-react";
import { Board } from "../../src-tauri/bindings/Board";

type PanelOptions = "moves" | "playerInfo"
const pieceIconColor = "#FFD6E0"; // Light pink, contrasts well with dark red

// Remove static mock taken arrays – now generated from board state
// const whiteTaken = [...]
// const blackTaken = [...]

// Map piece kind + color to an icon
function pieceToIcon(kind: string, color: string) {
    const commonProps = { color: pieceIconColor, size: 20 };
    switch (kind) {
        case "Pawn":
            return <ChessPawn {...commonProps} />;
        case "Knight":
            return <ChessKnight {...commonProps} />;
        case "Bishop":
            return <ChessBishop {...commonProps} />;
        case "Rook":
            return <ChessRook {...commonProps} />;
        case "Queen":
            return <ChessQueen {...commonProps} />;
        case "King":
            return <ChessKing {...commonProps} />;
        default:
            return null;
    }
}

interface Props {
    chessBoard: Board | null;
}
export function PveLeftPanel({ chessBoard }: Props) {
    const [selectedPanelOption, setSelectedPanelOption] = useState<PanelOptions>("playerInfo")

    const tabClass = (key: PanelOptions) => {
        const base = `${removeDefaultForButton} w-full py-2 text-sm tracking-wide transition-colors`;
        const selected = key === selectedPanelOption;
        return selected
            ? `${base} bg-secondary/15 text-secondary border-secondary/30`
            : `${base} text-secondary/70 hover:bg-secondary/10`;
    };

    return (
        <div className="flex flex-col left-panel w-[25%] h-full bg-primary/20 border-l border-secondary/30">
            <div className="py-5 px-4 border-b border-secondary/30">
                <h1 className="text-lg font-light text-secondary/90">Game Info</h1>
            </div>

            <div className="w-full border-b border-secondary/30">
                <div className="grid grid-cols-2 border-x border-secondary/30">
                    <button
                        onClick={() => setSelectedPanelOption("moves")}
                        className={tabClass("moves")}
                        style={{ borderRight: "1px solid rgba(254,205,190,0.2)" }}
                    >
                        Moves
                    </button>
                    <button
                        onClick={() => setSelectedPanelOption("playerInfo")}
                        className={tabClass("playerInfo")}
                    >
                        Info
                    </button>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto">
                {selectedPanelOption === "playerInfo" && (
                    <div className="text-secondary/80 text-sm">
                        {/* Black player info */}
                        <div className="border-b-2 border-secondary/20 py-4 px-4 gap-2">
                            <div className="w-full flex flex-row justify-between items-center mb-3">
                                <h1 className="text-base font-light">Black</h1>
                                <h1 className="text-xl font-medium">Player 1</h1>
                            </div>
                            <div className="w-full flex flex-row justify-between items-center mb-2">
                                <h1 className="text-sm font-normal">Elo</h1>
                                <h1 className="text-sm font-semibold">2100</h1>
                            </div>
                            <div className="w-full flex flex-col gap-2 items-start">
                                <h1 className="text-sm font-normal">
                                    Taken <span className="text-xs text-secondary/60">( +0.31 )</span>
                                </h1>
                                <div className="flex flex-row gap-2">
                                    {chessBoard?.ui.black_taken.map(([kind, color], idx) => (
                                        <span key={`black_taken_${idx}`} className="flex items-center">
                                            {pieceToIcon(kind, color)}
                                        </span>
                                    ))}
                                </div>
                            </div>
                        </div>
                        {/* White player info */}
                        <div className="border-b-2 border-secondary/20 py-4 px-4 gap-2">
                            <div className="w-full flex flex-row justify-between items-center mb-3">
                                <h1 className="text-base font-light">White</h1>
                                <h1 className="text-xl font-medium">Player 2</h1>
                            </div>
                            <div className="w-full flex flex-row justify-between items-center mb-2">
                                <h1 className="text-sm font-normal">Elo</h1>
                                <h1 className="text-sm font-semibold">1900</h1>
                            </div>
                            <div className="w-full flex flex-col gap-2 items-start">
                                <h1 className="text-sm font-normal">
                                    Taken <span className="text-xs text-secondary/60">( -1.31 )</span>
                                </h1>
                                <div className="flex flex-row gap-2">
                                    {chessBoard?.ui.white_taken.map(([kind, color], idx) => (
                                        <span key={`white_taken_${idx}`} className="flex items-center">
                                            {pieceToIcon(kind, color)}
                                        </span>
                                    ))}
                                </div>
                            </div>
                        </div>
                    </div>
                )}


                {selectedPanelOption === "moves" && chessBoard && (
                    <div className="text-secondary/80 text-sm w-[100%] flex flex-row py-4 px-2">
                        <div className="w-[20%] flex flex-col gap-4 pl-2">
                            {Array.from({
                                length:
                                    chessBoard.meta_data.move_list.length % 2 === 0
                                        ? chessBoard.meta_data.move_list.length / 2
                                        : Math.floor(chessBoard.meta_data.move_list.length / 2) + 1
                            }).map((_, idx) => (
                                <span key={idx}>{idx + 1}.</span>
                            ))}
                        </div>
                        <div className="w-[40%] flex flex-col gap-4 pl-2">
                            {chessBoard.meta_data.move_list.filter((_, idx) => idx % 2 === 0).map((mv, idx) => (
                                <div key={idx}>{mv.uci}</div>
                            ))}
                        </div>
                        <div className="w-[40%] flex flex-col gap-4 pl-2">
                            {chessBoard.meta_data.move_list.filter((_, idx) => idx % 2 === 1).map((mv, idx) => (
                                <div key={idx}>{mv.uci}</div>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
}
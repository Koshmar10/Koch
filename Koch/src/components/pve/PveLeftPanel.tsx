import { useState } from "react"
import { removeDefaultForButton } from "../../App";
import { ChessPawn, ChessKnight, ChessBishop, ChessRook, ChessQueen, ChessKing } from "lucide-react";
import { Board } from "../../../src-tauri/bindings/Board";
import { PieceType } from "../../../src-tauri/bindings/PieceType";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";

type PanelOptions = "moves" | "playerInfo"
const pieceIconColor = "#FFD6E0"; // Light pink, contrasts well with dark red

// Remove static mock taken arrays – now generated from board state
// const whiteTaken = [...]
// const blackTaken = [...]

// Map piece kind + color to an icon
function pieceToIcon(kind: string, _color: string) {
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
    blackTaken: [PieceType, PieceColor][];
    whiteTaken: [PieceType, PieceColor][];
    opening?: string;
    moveList: string[];
}
export function PveLeftPanel({ moveList, opening }: Props) {
    const [selectedPanelOption, setSelectedPanelOption] = useState<PanelOptions>("playerInfo")

    const tabClass = (key: PanelOptions) => {
        const base = `${removeDefaultForButton} w-full py-2 text-sm tracking-wide transition-colors`;
        const selected = key === selectedPanelOption;
        return selected
            ? `${base} bg-secondary/15 text-secondary border-secondary/30`
            : `${base} text-secondary/70 hover:bg-secondary/10`;
    };

    return (
        <div className="flex flex-col left-panel w-[25%] h-full bg-card-dark/45 border-l border-border-dark">
            <div className="py-5 px-4 border-b border-secondary/30">
                <h1 className="text-lg font-light text-foreground-dark">Game Info</h1>
            </div>


            <div className="flex-1 overflow-y-auto">

                <div className="text-secondary/80 text-sm w-full flex flex-col py-4 px-2 gap-4 border-b-2 border-border-dark">
                    <div className="text-base font-medium text-foreground-dark">Opening</div>
                    <div className="flex flex-row items-center gap-2">
                        {/* Placeholder for opening name, replace with actual prop or state if available */}
                        <span className="text-foreground-dark">{opening ? opening : "Unknown Opening"}</span>
                    </div>
                </div>
                <div className="text-secondary/80 text-sm w-full flex flex-col py-4 px-2 gap-4">
                    <div className="text-base font-medium text-foreground-dark">Move List</div>
                    <div className="flex flex-col gap-1">
                        {moveList.length !== 0 && Array.from({
                            length:
                                moveList.length % 2 === 0
                                    ? moveList.length / 2
                                    : Math.floor(moveList.length / 2) + 1
                        }).map((_, idx) => {
                            const whiteMove = moveList[idx * 2]?.split('|')[1] || "";
                            const blackMove = moveList[idx * 2 + 1]?.split('|')[1] || "";
                            return (
                                <div
                                    key={idx}
                                    className="flex flex-row items-center rounded-md px-2 py-1 hover:bg-primary/60 transition-colors duration-200 cursor-pointer"
                                >
                                    <span className="w-8 text-sm font-light text-muted">{idx + 1}.</span>
                                    <span className="w-24 text-sm font-medium text-foreground-dark">{whiteMove}</span>
                                    <span className="w-24 text-sm font-medium text-foreground-dark">{blackMove}</span>
                                </div>
                            );
                        })}
                    </div>

                </div>

            </div>
        </div>
    )
}
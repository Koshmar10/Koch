import { Board } from "../../src-tauri/bindings/Board";
import { PieceColor } from "../../src-tauri/bindings/PieceColor";
import white_pawn from "../assets/pieces/white_pawn.png";
import white_knight from "../assets/pieces/white_knight.png";
import white_bishop from "../assets/pieces/white_bishop.png";
import white_rook from "../assets/pieces/white_rook.png";
import white_queen from "../assets/pieces/white_queen.png";
import white_king from "../assets/pieces/white_king.png";
import black_pawn from "../assets/pieces/black_pawn.png";
import black_knight from "../assets/pieces/black_knight.png";
import black_bishop from "../assets/pieces/black_bishop.png";
import black_rook from "../assets/pieces/black_rook.png";
import black_queen from "../assets/pieces/black_queen.png";
import black_king from "../assets/pieces/black_king.png";
import { PieceMoves } from "../../src-tauri/bindings/PieceMoves";
import { ChessPiece } from "../../src-tauri/bindings/ChessPiece";
import { GameController } from "../../src-tauri/bindings/GameController";
import { useMemo } from "react";
import { PieceType } from "../../src-tauri/bindings/PieceType";

const PIECE_IMAGES: Record<PieceColor, Record<PieceType, string>> = {
    White: {
        Pawn: white_pawn,
        Knight: white_knight,
        Bishop: white_bishop,
        Rook: white_rook,
        Queen: white_queen,
        King: white_king,
    },
    Black: {
        Pawn: black_pawn,
        Knight: black_knight,
        Bishop: black_bishop,
        Rook: black_rook,
        Queen: black_queen,
        King: black_king,
    },
};
function getImage(color: PieceColor, kind: PieceType): string {
    return PIECE_IMAGES[color][kind];
}

interface Props {
    board: Board | null,
    playerColor: PieceColor,
    squareSize: number
}
export function AnalyzerBoard({ squareSize, board, playerColor }: Props) {
    const activePieces = useMemo(() => {
        if (!board) return [];
        const pieces: Array<{ piece: ChessPiece, r: number, c: number }> = [];

        board.squares.forEach((row, rIdx) => {
            row.forEach((p, cIdx) => {
                if (p) {
                    pieces.push({ piece: p, r: rIdx, c: cIdx });
                }
            });
        });
        return pieces;
    }, [board]);
    return (
        <div className="relative inline-block select-none" style={{ width: squareSize * 8, height: squareSize * 8 }}>

            {/* LAYER 1: The Grid (Background & Clicks) */}
            <div className="grid grid-cols-8 grid-rows-8 w-fit absolute top-0 left-0 z-10">
                {Array.from({ length: 8 }).map((_, rowIdx) =>
                    Array.from({ length: 8 }).map((_, colIdx) => {
                        let pvRowIdx = rowIdx;
                        let pvColIdx = colIdx;
                        let whr = 7;
                        let bhr = 0
                        if (playerColor == "Black") {
                            pvRowIdx = 7 - rowIdx;
                            pvColIdx = 7 - pvColIdx;
                            whr = 0;
                            bhr = 7;
                        }
                        // We still need to know if a piece is logically here for click handling
                        let piece = board?.squares[pvRowIdx][pvColIdx] || null;

                        let promotionTarget = piece?.kind == "Pawn" && (
                            (piece.color == "White" && pvRowIdx == bhr)
                            || (piece.color == "Black" && pvRowIdx == whr)
                        );



                        const isLight = (rowIdx + colIdx) % 2 === 0;
                        const lightColor = "#f0d9b5";
                        const darkColor = "#b58863";
                        const selectedLight = "#f7b64c";
                        const selectedDark = "#e08a2e";
                        const checkLight = "#ffcccc";
                        const checkDark = "#ff6666";

                        let baseColor = isLight ? lightColor : darkColor;

                        const squareColor = baseColor;

                        return (
                            <div
                                key={`${pvRowIdx}-${pvColIdx}`}
                                className="relative flex items-center justify-center"
                                style={{
                                    width: squareSize,
                                    height: squareSize,
                                    backgroundColor: squareColor
                                }}

                            >


                            </div>
                        );
                    })
                )}
            </div>

            {/* LAYER 2: The Pieces (Animated) */}
            <div className="absolute top-0 left-0 w-full h-full z-20 pointer-events-none">
                {activePieces.map(({ piece, r, c }) => {
                    // Calculate visual position based on POV
                    let visualRow = r;
                    let visualCol = c;

                    if (playerColor == "Black") {
                        visualRow = 7 - r;
                        visualCol = 7 - c;
                    }

                    return (
                        <img
                            key={piece.id} // CRITICAL: Stable ID allows React to animate the same element
                            src={getImage(piece.color, piece.kind)}
                            alt=""
                            className="absolute transition-all duration-200 ease-in-out"
                            style={{
                                width: squareSize * 0.8,
                                height: squareSize * 0.8,
                                // Center the piece in the square
                                transform: `translate(${visualCol * squareSize + (squareSize * 0.1)}px, ${visualRow * squareSize + (squareSize * 0.1)}px)`,
                            }}
                            draggable={false}
                        />
                    );
                })}
            </div>

            {/* Rank annotations */}
            {Array.from({ length: 8 }).map((_, rowIdx) => {
                const rank = 8 - rowIdx;
                return (
                    <span
                        key={`rank-${rank}`}
                        className="absolute text-[11px] font-semibold text-secondary/80 pointer-events-none select-none"
                        style={{
                            left: -16,
                            top: rowIdx * squareSize + 4
                        }}
                    >
                        {rank}
                    </span>
                );
            })}

            {/* File annotations */}
            {Array.from({ length: 8 }).map((_, colIdx) => {
                const file = String.fromCharCode(65 + colIdx);
                return (
                    <span
                        key={`file-${file}`}
                        className="absolute text-[11px] font-semibold text-secondary/80 pointer-events-none select-none"
                        style={{
                            top: squareSize * 8 + 2,
                            left: colIdx * squareSize + squareSize / 2 - squareSize / 2 + 8
                        }}
                    >
                        {file.toLocaleLowerCase()}
                    </span>
                );
            })}
        </div>
    )
}
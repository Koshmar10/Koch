import { Dispatch, SetStateAction, useEffect, useState, useMemo } from "react";
import { Board } from "../../src-tauri/bindings/Board";
import { PieceType } from "../../src-tauri/bindings/PieceType";
import { PieceColor } from "../../src-tauri/bindings/PieceColor";
import { invoke } from '@tauri-apps/api/core'
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

interface Props {
    chessBoard: Board | null;
    setChessBoard: (val: Board | null) => void;
    gameState: GameController | null;
    setGameState: Dispatch<SetStateAction<GameController | null>>
}

function getImage(color: PieceColor, kind: PieceType): string {
    return PIECE_IMAGES[color][kind];
}

export function ChessBoard({ setGameState, gameState, chessBoard, setChessBoard }: Props) {
    const [selectedPiece, setSelectedPiece] = useState<ChessPiece | null>(null);
    const [selectedPieceMoves, setSelectedPieceMoves] = useState<PieceMoves | null>(null);
    const [selectedDestSquare, setSelectedDestSquare] = useState<[number, number] | null>(null);
    const [gameOverPopupOpen, setGameOverPopupOpen] = useState<boolean>(false);
    const [promotionPending, setPromotionPending] = useState<boolean>(false);
    const [gameSaved, setGameSaved] = useState<boolean>(false)
    useEffect(() => {
        if (!gameState || !chessBoard) return;
        if (promotionPending) return;
        const timer = setTimeout(async () => {
            try {
                const [new_board, new_controller] =
                    await invoke<[Board, GameController]>('update_gameloop');

                // Only update if something actually changed to prevent micro-jitters
                // (Optional: add deep comparison here if needed)
                setChessBoard(new_board);
                setGameState(new_controller);


            } catch (e) {
                console.error("update_gameloop failed", e);
            }
        }, 400); // Increased slightly to allow animation to finish before next poll
        // Game termination notification (runs once)/


        const lostby = setTimeout(async () => {
            try {
                if (gameState.game_over) {
                    setGameOverPopupOpen(true);
                }


            } catch (e) {
                console.error("update_gameloop failed", e);
            }
        }, 400); // Increased slightly to allow animation to finish before next poll
        // Game termination notification (runs once)/
        return () => { clearTimeout(timer); clearTimeout(lostby); }

    }, [chessBoard, gameState]); // Removed specific dependency on turn to ensure loop continues

    const handleSquareClick = async (rowIdx: number, colIdx: number, piece: ChessPiece | null) => {
        if (promotionPending) { return; }
        let move_succeded = false;
        if (!chessBoard) return;
        if (selectedPiece && piece && piece.id == selectedPiece.id) return;
        if (gameState && !selectedPiece && piece && piece.color != gameState.player) return;

        if (selectedPiece) {
            let dest: [number, number] | null = null;

            if (piece) {
                if (piece.color !== selectedPiece.color) {
                    dest = [rowIdx, colIdx];
                } else {
                    // switch to new friendly piece
                    setSelectedPiece(piece);
                    setSelectedPieceMoves(chessBoard.move_cache[piece.id] || null);
                    setSelectedDestSquare(null);
                    return;
                }
            } else {
                dest = [rowIdx, colIdx];
            }

            if (!dest) return;

            // Optimistic UI update could go here for instant feedback

            const moveResult = await invoke<Board | null>('try_move', {
                srcSquare: selectedPiece.position,
                destSquare: dest,
            });
            if (moveResult) {
                // NEW: Detect if the move caused a promotion scenario
                const movedPiece = moveResult.squares[dest[0]][dest[1]];
                if (movedPiece && movedPiece.kind === "Pawn") {
                    // White promotes at row 0, Black at row 7 (internal board coordinates)
                    const isWhitePromo = movedPiece.color === "White" && dest[0] === 0;
                    const isBlackPromo = movedPiece.color === "Black" && dest[0] === 7;

                    if (isWhitePromo || isBlackPromo) {
                        setPromotionPending(true);
                    }
                }
                setChessBoard(moveResult);
            }

            setSelectedDestSquare(null);
            setSelectedPiece(null);
            setSelectedPieceMoves(null);
            return;
        } else {
            if (!piece) {
                setSelectedDestSquare(null);
                setSelectedPiece(null);
                setSelectedPieceMoves(null);
                return;
            }
            if (piece.color === chessBoard.turn) {
                setSelectedPiece(piece);
                setSelectedPieceMoves(chessBoard.move_cache[piece.id] || null);
                setSelectedDestSquare(null);
            }
        }
    };

    const squareSize = 60; // px

    // 1. Flatten the board to get a list of pieces with their coordinates
    const activePieces = useMemo(() => {
        if (!chessBoard) return [];
        const pieces: Array<{ piece: ChessPiece, r: number, c: number }> = [];

        chessBoard.squares.forEach((row, rIdx) => {
            row.forEach((p, cIdx) => {
                if (p) {
                    pieces.push({ piece: p, r: rIdx, c: cIdx });
                }
            });
        });
        return pieces;
    }, [chessBoard]);

    return (
        <div className="relative">
            {
                !gameSaved && gameOverPopupOpen && (
                    <div className="absolute inset-0 z-40 flex items-center justify-center">
                        <div
                            className="absolute inset-0 bg-black/30 backdrop-blur-sm"
                            onClick={() => setGameOverPopupOpen(false)}
                        />
                        <div className="relative w-80 rounded-lg bg-dark/90 shadow-lg p-4 space-y-4">
                            <div className="text-center space-y-2">
                                <h1 className="text-xl font-bold">Game Over</h1>
                                {/* OLD: <h2 className="text-sm">Winner {chessBoard?.turn}</h2> */}
                                {chessBoard && (
                                    <h2 className="text-sm">
                                        Winner {chessBoard.turn === "White" ? "Black" : "White"}
                                    </h2>
                                )}
                                {/* Alternatively use result mapping */}
                                {/* <h2 className="text-sm">
                                    {(() => {
                                        switch (chessBoard?.meta_data.result) {
                                            case "WhiteWin": return "Winner White";
                                            case "BlackWin": return "Winner Black";
                                            case "Draw": return "Draw";
                                            default: return "";
                                        }
                                    })()}
                                </h2> */}
                                <h4>Game ended by: {gameState?.lost_by}</h4>
                            </div>
                            <div className="flex flex-row justify-between gap-2">
                                <button className="px-3 py-1 rounded bg-accent/80 hover:bg-accent"
                                    onClick={async (e) => {
                                        e.preventDefault();
                                        let res = await invoke('game_into_db');
                                        setGameOverPopupOpen(false);
                                        setGameSaved(true);
                                    }}>Save</button>
                                <button className="px-3 py-1 rounded bg-accent/80 hover:bg-accent"
                                    onClick={async (e) => {
                                        e.preventDefault()
                                        const [board, game] = await invoke<[Board, GameController]>('start_game');
                                        console.log("Player info:", game.player);
                                        console.log("Enemy: ", game.enemy);
                                        setChessBoard(board);
                                        setGameState(game);
                                        setGameOverPopupOpen(false);
                                        setSelectedPiece(null);
                                        setSelectedPieceMoves(null);

                                    }}>New Game</button>
                                <button className="px-3 py-1 rounded bg-accent/80 hover:bg-accent">Analyze</button>
                            </div>
                            <button
                                className="absolute top-2 right-2 text-xs px-2 py-1 rounded bg-black/30 hover:bg-black/50"
                                onClick={() => setGameOverPopupOpen(false)}
                            >
                                Close
                            </button>
                        </div>
                    </div>
                )
            }
            <div className="relative inline-block select-none" style={{ width: squareSize * 8, height: squareSize * 8 }}>

                {/* LAYER 1: The Grid (Background & Clicks) */}
                <div className="grid grid-cols-8 grid-rows-8 w-fit absolute top-0 left-0 z-10">
                    {Array.from({ length: 8 }).map((_, rowIdx) =>
                        Array.from({ length: 8 }).map((_, colIdx) => {
                            let pvRowIdx = rowIdx;
                            let pvColIdx = colIdx;
                            let whr = 7;
                            let bhr = 0
                            if (gameState && gameState.player == "Black") {
                                pvRowIdx = 7 - rowIdx;
                                pvColIdx = 7 - pvColIdx;
                                whr = 0;
                                bhr = 7;
                            }
                            // We still need to know if a piece is logically here for click handling
                            let piece = chessBoard?.squares[pvRowIdx][pvColIdx] || null;

                            let promotionTarget = piece?.kind == "Pawn" && (
                                (piece.color == "White" && pvRowIdx == bhr)
                                || (piece.color == "Black" && pvRowIdx == whr)
                            );

                            const isSelected =
                                !!selectedPiece &&
                                selectedPiece.position[0] === pvRowIdx &&
                                selectedPiece.position[1] === pvColIdx;

                            const isQuietMove =
                                selectedPieceMoves?.quiet_moves.some(([r, c]) => r === pvRowIdx && c === pvColIdx);

                            const isCaptureMove =
                                selectedPieceMoves?.capture_moves.some(([r, c]) => r === pvRowIdx && c === pvColIdx);

                            const isLight = (rowIdx + colIdx) % 2 === 0;
                            const lightColor = "#f0d9b5";
                            const darkColor = "#b58863";
                            const selectedLight = "#f7b64c";
                            const selectedDark = "#e08a2e";
                            const checkLight = "#ffcccc";
                            const checkDark = "#ff6666";

                            let baseColor = isLight ? lightColor : darkColor;
                            const highlightColor = isLight ? selectedLight : selectedDark;
                            if (piece && piece.kind == "King" && piece.color == gameState?.player && gameState.in_check && chessBoard?.turn == gameState.player) {
                                baseColor = isLight ? checkLight : checkDark;
                            }
                            const squareColor = isSelected ? highlightColor : baseColor;

                            return (
                                <div
                                    key={`${pvRowIdx}-${pvColIdx}`}
                                    className="relative flex items-center justify-center"
                                    style={{
                                        width: squareSize,
                                        height: squareSize,
                                        backgroundColor: squareColor
                                    }}
                                    onClick={() => { if (promotionTarget) return; handleSquareClick(pvRowIdx, pvColIdx, piece) }}
                                >
                                    {
                                        promotionTarget && (
                                            <div className="absolute top-[-50px] left-[-40px] bg-primary/60 rounded-xl z-0">
                                                {
                                                    <div className="flex gap-1 p-1 rounded w-[150px]">
                                                        {Object.entries(PIECE_IMAGES[piece!.color])
                                                            .filter(([t]) => t !== "Pawn" && t !== "King")
                                                            .map(([t, src]) => (
                                                                <img
                                                                    key={t}
                                                                    src={src}
                                                                    alt={`${piece!.color} ${t}`}
                                                                    className="w-8 h-8 cursor-pointer hover:scale-105 transition"
                                                                    draggable={false}
                                                                    onClick={async (e) => {
                                                                        e.preventDefault()
                                                                        let new_board = await invoke<Board>('promote_pawn', {
                                                                            pos: [pvRowIdx, pvColIdx],
                                                                            kind: t
                                                                        })
                                                                        setChessBoard(new_board);
                                                                        setPromotionPending(false);
                                                                    }}
                                                                />
                                                            ))}
                                                    </div>
                                                }
                                            </div>
                                        )
                                    }
                                    {/* Markers for moves */}
                                    {isQuietMove && !piece && (
                                        <div className="absolute w-4 h-4 rounded-full bg-gray-700/70" />
                                    )}
                                    {isCaptureMove && piece && (
                                        <div className="absolute w-8 h-8 rounded-full border-4 border-red-600 pointer-events-none" />
                                    )}
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

                        if (gameState && gameState.player == "Black") {
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
        </div>
    );
}
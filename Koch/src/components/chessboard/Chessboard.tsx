import { useEffect, useMemo, useState } from "react";
import { Board } from "../../../src-tauri/bindings/Board";
import { BoardSquare } from "./BoardSquare";
import { ChessPiece } from "../../../src-tauri/bindings/ChessPiece";
import { Piece } from "./Piece";
import { ArrowData, ArrowLayer } from "../ArrowLayer";
import { PieceMoves } from "../../../src-tauri/bindings/PieceMoves";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";
import { getImage } from "./utils";
import { PlayerClock } from "./PlayerClock";
import { PlayerCard } from "./PlayerCard";
import { PieceLayer } from "./PieceLayer";
const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const RANKS = ['8', '7', '6', '5', '4', '3', '2', '1'];
const PROMOTION_PIECES: PieceType[] = ["Queen", "Rook", "Bishop", "Knight"];

const DEFAULT_TIME = 1000 * 60 * 10; // 10 minutes

export interface RenderedPiece {
    piece: ChessPiece;
    r: number;
    c: number;
    to_render: boolean; // NEW: Controls visibility instead of array removal
}
export interface GhostPiece {
    kind: PieceType,
    color: PieceColor,
    r: number,
    c: number,
}

interface ChessBoardProps {
    squareSize: number;
    board: Board;
    onMove?: (from: [number, number], to: [number, number], promotion?: string) => void;
    lastMove?: [number[], number[]] | null;
    flipped?: boolean;
    isInteractive?: boolean;
    whiteClock?: string; // <- NEW
    blackPlayer?: string;
    whitePlayer?: string;
    blackClock?: string; // <- NEW
    suggestion?: ArrowData | null;
    threat?: ArrowData | null;
    tintEnabled?: boolean; // when true show influence tint; when true do not render last-move/selection highlights
    hoverTarget?: [number, number] | null;
    ghostPieces?: GhostPiece[];
    ghostArrows?: ArrowData[];
    user?: PieceColor;
}

export function ChessBoard({
    squareSize,
    board,
    onMove,
    lastMove,
    flipped = false,
    isInteractive = true,
    whiteClock,
    blackClock,
    whitePlayer,
    blackPlayer,
    suggestion = null,
    threat = null,
    tintEnabled = false,
    hoverTarget = null,
    ghostPieces = [],
    ghostArrows = [],
    user,

}: ChessBoardProps) {

    const [selectedSquare, setSelectedSquare] = useState<number[] | null>(null)
    const [selectedMoves, setSelectedMoves] = useState<PieceMoves | null>(null)
    const [arrows, setArrows] = useState<ArrowData[]>([]);
    const [startArrow, setStartArrow] = useState<number[] | null>(null);
    const [lastMoveMade, setLastMoveMade] = useState<[number[], number[]] | null>(lastMove || null);
    // State to handle promotion selection
    const [promotionMove, setPromotionMove] = useState<{ from: [number, number], to: [number, number], color: PieceColor } | null>(null);

    // 1. Local state for pieces
    const [optimisticPieces, setOptimisticPieces] = useState<RenderedPiece[]>([]);

    // 2. Sync local state whenever the backend 'board' prop updates
    useEffect(() => {
        if (!board) return;
        setSelectedMoves(null);
        setSelectedSquare(null);
        setOptimisticPieces(prev => {
            // Create a map of existing pieces for stable updates
            const nextPiecesMap = new Map<number, RenderedPiece>();

            // Initialize map with previous pieces, defaulting to not rendered (captured)
            prev.forEach(p => {
                nextPiecesMap.set(p.piece.id, { ...p, to_render: false });
            });

            // Process current board state
            board.squares.forEach((row, rIdx) => {
                row.forEach((p, cIdx) => {
                    if (p) {
                        const existing = nextPiecesMap.get(p.id);
                        if (existing) {
                            // Update existing piece: position and visibility
                            nextPiecesMap.set(p.id, {
                                ...existing,
                                piece: p, // Update piece data (e.g. type change)
                                r: rIdx,
                                c: cIdx,
                                to_render: true
                            });
                        } else {
                            // Add new piece (e.g. promotion or initial load)
                            nextPiecesMap.set(p.id, {
                                piece: p,
                                r: rIdx,
                                c: cIdx,
                                to_render: true
                            });
                        }
                    }
                });
            });

            return Array.from(nextPiecesMap.values());
        });
    }, [board]);

    // Highlight squares from parent-provided UCI move
    useEffect(() => {
        if (!lastMove) {
            setLastMoveMade(null);
            return;
        }
        setLastMoveMade(lastMove);
    }, [lastMove]);

    const handleContextMenu = (e: React.MouseEvent) => {
        e.preventDefault();
    };

    const isLegalMove = (r: number, c: number): boolean => {
        if (!selectedMoves) return false;
        return false
            || selectedMoves.capture_moves.some((m) => m[0] === r && m[1] === c)
            || selectedMoves.quiet_moves.some((m) => m[0] === r && m[1] === c)
    }

    const selectSquare = (r: number, c: number) => {
        ///if (!isInteractive) return;
        const piece = board.squares[r][c];
        if (!piece) return;
        if (user ? piece.color !== user : false) return;
        setSelectedSquare([r, c]);
        setSelectedMoves(board.move_cache[piece.id] || null);
    }
    const deselectSquare = () => {
        setSelectedSquare(null);
        setSelectedMoves(null);
    }

    const handlePromotionSelect = (pieceType: PieceType) => {
        if (!promotionMove) return;
        const { from, to } = promotionMove;

        // Optimistic update with the promoted piece
        setOptimisticPieces(prev => {
            return prev.map(p => {
                // Hide captured piece at destination
                if (p.r === to[0] && p.c === to[1] && p.to_render) {
                    return { ...p, to_render: false };
                }
                // Move and promote the pawn
                if (p.r === from[0] && p.c === from[1] && p.to_render) {
                    return {
                        ...p,
                        r: to[0],
                        c: to[1],
                        piece: { ...p.piece, kind: pieceType }
                    };
                }
                return p;
            });
        });

        deselectSquare();
        setPromotionMove(null);

        if (onMove) {
            onMove(from, to, pieceType);
        }
    };

    const onSquareMouseDown = (r: number, c: number, e: React.MouseEvent) => {
        if (e.button === 0) {
            setArrows([]);

            // 3. Handle Move Logic
            if (selectedSquare) {
                const [fromR, fromC] = selectedSquare;

                // If clicking the same square, deselect
                if (fromR === r && fromC === c) {
                    deselectSquare()
                    return;
                }

                if (isLegalMove(r, c)) {
                    // --- PROMOTION CHECK START ---
                    const movingPiece = board.squares[fromR][fromC];
                    if (movingPiece && movingPiece.kind === "Pawn") {
                        const isWhitePromotion = movingPiece.color === "White" && r === 0;
                        const isBlackPromotion = movingPiece.color === "Black" && r === 7;

                        if (isWhitePromotion || isBlackPromotion) {
                            setPromotionMove({
                                from: [fromR, fromC],
                                to: [r, c],
                                color: movingPiece.color
                            });
                            return; // Stop here, wait for user selection
                        }
                    }
                    // --- PROMOTION CHECK END ---

                    // --- OPTIMISTIC UPDATE START ---
                    setOptimisticPieces(prev => {
                        return prev.map(p => {
                            // A. Capture: Hide piece at destination
                            if (p.r === r && p.c === c && p.to_render) {
                                return { ...p, to_render: false };
                            }
                            // B. Move: Update coordinates of selected piece
                            if (p.r === fromR && p.c === fromC && p.to_render) {
                                return { ...p, r: r, c: c };
                            }
                            return p;
                        });
                    });
                    // --- OPTIMISTIC UPDATE END ---

                    // Clear selection
                    deselectSquare()
                    setLastMoveMade([[fromR, fromC], [r, c]]);
                    if (onMove) {
                        onMove([fromR, fromC], [r, c]);
                    }


                    return;
                }
            }

            // Normal selection logic (if not moving)
            const piece = board.squares[r][c];
            const piece_color = piece?.color;

            // Do not select empty squares
            if (!piece) {
                deselectSquare();
                return;
            }

            // Ignore clicks on opponent's pieces
            if (piece_color !== board.turn) {
                return;
            }
            console.log(r, c);
            selectSquare(r, c)
        }
        if (e.button === 2) {
            setStartArrow([r, c]);
        }
    };

    const onSquareMouseUp = (r: number, c: number, e: React.MouseEvent) => {
        if (e.button === 2) {
            if (startArrow) {
                if (startArrow[0] !== r || startArrow[1] !== c) {
                    setArrows(prev => [
                        ...prev,
                        { from: `${startArrow[0]}-${startArrow[1]}`, to: `${r}-${c}`, color: "orange", type: "user" },
                    ]);
                } else {
                    setArrows([]);
                }
            }
            setStartArrow(null);
        }
    };

    // Helper to get visual coordinates based on flip state


    // Helper to get logical coordinates from visual index
    const getLogicalCoords = (visualRow: number, visualCol: number) => {
        return flipped ? [7 - visualRow, 7 - visualCol] : [visualRow, visualCol];
    };

    const isInCheck = (r: number, c: number, color?: PieceColor): boolean => {
        let in_check = false;
        let piece = board.squares[r][c];
        if (piece === null) return in_check;
        if (piece.kind !== "King") return in_check;
        if (piece.color !== color) return in_check;
        // Build a map of piece ID -> color for quick lookup
        const idToColor = new Map<number, PieceColor>();
        for (let rr = 0; rr < 8; rr++) {
            for (let cc = 0; cc < 8; cc++) {
                const p = board.squares[rr][cc];
                if (p) idToColor.set(p.id, p.color);
            }
        }

        // Check if any opponent piece has a capture move to [r, c]
        for (const [idStr, moves] of Object.entries(board.move_cache)) {
            const id = Number(idStr);
            const attackerColor = idToColor.get(id);
            if (!attackerColor || attackerColor === color) continue;
            if (moves) {

                if (moves.capture_moves.some(([mr, mc]) => mr === r && mc === c)) {
                    in_check = true;
                    break;
                }
            }
        }
        return in_check;

    }
    return (
        <div className="flex flex-col gap-2">

            {/* Top Player Card (Opponent) */}
            <PlayerCard
                display={true}
                color={flipped ? "White" : "Black"}
                player={
                    flipped
                        ? (whitePlayer !== undefined ? whitePlayer : board.meta_data.white_player_name)
                        : (blackPlayer !== undefined ? blackPlayer : board.meta_data.black_player_name)
                }
                isTurn={board.turn === (flipped ? "White" : "Black")}
                rating={flipped ? board.meta_data.white_player_elo : board.meta_data.black_player_elo}
                clock={flipped ? whiteClock : blackClock} // <- pass appropriate clock
            />

            <div className="relative inline-block" onContextMenu={handleContextMenu}>
                {/* <div className="absolute top-[-30px] left-0 ">{board.game_phase}</div> */}
                {/* Board squares */}
                <div
                    className="grid relative z-0"
                    style={{
                        gridTemplateColumns: `repeat(8, ${squareSize}px)`,
                        gridTemplateRows: `repeat(8, ${squareSize}px)`,
                        width: squareSize * 8,
                        height: squareSize * 8,
                    }}
                >
                    {Array.from({ length: 64 }, (_, i) => {
                        const visualRow = Math.floor(i / 8);
                        const visualCol = i % 8;
                        const [row, col] = getLogicalCoords(visualRow, visualCol);
                        const isDark = (visualRow + visualCol) % 2 === 1;

                        // When tint is enabled we intentionally hide selected/last-move highlights
                        const isSelected = tintEnabled
                            ? false
                            : (
                                (!!selectedSquare && selectedSquare[0] === row && selectedSquare[1] === col) ||
                                (!!lastMoveMade &&
                                    ((lastMoveMade[0][0] === row && lastMoveMade[0][1] === col) ||
                                        (lastMoveMade[1][0] === row && lastMoveMade[1][1] === col)))
                            );

                        let tintColor: string | undefined = undefined;
                        // Compute influence tints only when enabled
                        if (tintEnabled) {
                            let whiteInfluence = 0;
                            let blackInfluence = 0;
                            for (const [idStr, moves] of Object.entries(board.move_cache)) {
                                const id = Number(idStr);
                                const rendered = optimisticPieces.find(p => p.piece.id === id);
                                if (!rendered) continue;
                                const owner = rendered.piece.color;
                                if (moves && Array.isArray(moves.attacks) && moves.attacks.some(([mr, mc]) => mr === row && mc === col)) {
                                    if (owner === "White") whiteInfluence += 1;
                                    else blackInfluence += 1;
                                }
                            }

                            if (whiteInfluence === 0 && blackInfluence === 0) {
                                tintColor = undefined;
                            } else {
                                // compute base alpha from influence intensity
                                const computeAlpha = (diff: number, baseMin = 0.12, baseMaxDelta = 0.38) => {
                                    const intensity = Math.min(diff, 7) / 7; // 0..1
                                    return baseMin + intensity * baseMaxDelta; // numeric alpha
                                };

                                // adjust alpha depending on whether the underlying square is dark or light:
                                // light squares -> slightly more opaque (makes tint pop)
                                // dark squares  -> slightly less opaque (preserve base color)
                                const opacityFactor = isDark ? 0.72 : 1.25;

                                if (whiteInfluence > blackInfluence) {
                                    const diff = whiteInfluence - blackInfluence;
                                    let alpha = computeAlpha(diff);
                                    alpha = Math.max(0.03, Math.min(1, alpha * opacityFactor));
                                    tintColor = `rgba(37,99,235,${alpha.toFixed(3)})`; // blue for White control
                                } else if (blackInfluence > whiteInfluence) {
                                    const diff = blackInfluence - whiteInfluence;
                                    let alpha = computeAlpha(diff);
                                    alpha = Math.max(0.03, Math.min(1, alpha * opacityFactor));
                                    tintColor = `rgba(220,38,38,${alpha.toFixed(3)})`; // red for Black control
                                } else {
                                    // equal non-zero -> contested/purple, slightly subtler
                                    const intensity = Math.min(whiteInfluence, 7);
                                    let alpha = computeAlpha(intensity, 0.10, 0.35);
                                    alpha = Math.max(0.03, Math.min(1, alpha * opacityFactor));
                                    tintColor = `rgba(128,0,128,${alpha.toFixed(3)})`; // contested -> purple
                                }
                            }
                        }

                        // if tint is disabled, ensure tintColor is undefined so base UI highlights render
                        // (tintColor already undefined when tintEnabled=false)
                        // Annotation Logic
                        const isLeftEdge = visualCol === 0;
                        const isBottomEdge = visualRow === 7;
                        const rankLabel = isLeftEdge ? (flipped ? visualRow + 1 : 8 - visualRow) : null;
                        const fileLabel = isBottomEdge ? (flipped ? FILES[7 - visualCol] : FILES[visualCol]) : null;

                        return (
                            <div

                                className="relative p-0 m-0"
                                style={{ width: squareSize, height: squareSize }}
                            >
                                <BoardSquare
                                    // key is now on the wrapper div
                                    key={`${row}-${col}`}
                                    dark={isDark}
                                    selected={isSelected}
                                    tint={tintColor}
                                    size={squareSize}
                                    attackMove={false}
                                    hover={hoverTarget ? hoverTarget[0] == row && hoverTarget[1] == col : false}
                                    inCheck={isInCheck(row, col, board.squares[row][col]?.color)}
                                    captureMove={!!selectedMoves && selectedMoves.capture_moves.some((m) => m[0] === row && m[1] === col)}
                                    quietMove={!!selectedMoves && selectedMoves.quiet_moves.some((m) => m[0] === row && m[1] === col)}
                                    handleMouseDown={(e) => onSquareMouseDown(row, col, e)}
                                    handleMouseUp={(e) => onSquareMouseUp(row, col, e)}
                                />

                                {/* Rank Annotation (Top-Left) */}
                                {rankLabel && (
                                    <span className={`absolute top-0.5 left-1 text-[10px] font-bold select-none pointer-events-none ${isDark ? "text-white/60" : "text-black/60"}`}>
                                        {rankLabel}
                                    </span>
                                )}

                                {/* File Annotation (Bottom-Right) */}
                                {fileLabel && (
                                    <span className={`absolute bottom-0 right-1 text-[10px] font-bold select-none pointer-events-none ${isDark ? "text-white/60" : "text-black/60"}`}>
                                        {fileLabel}
                                    </span>
                                )}
                            </div>
                        );
                    })}
                </div>

                {/* Pieces layer on top */}
                <PieceLayer flipped={flipped} optimisticPieces={optimisticPieces} squareSize={squareSize} ghostPieces={ghostPieces} />
                <ArrowLayer
                    arrows={arrows}
                    suggestion={suggestion}
                    threat={threat}
                    ghostArrows={ghostArrows}
                    isFlipped={flipped}
                    squareSize={squareSize}
                />

                {/* Promotion Modal */}
                {promotionMove && (
                    <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/40 rounded-sm backdrop-blur-[1px]">
                        <div className="bg-zinc-900/90 p-3 rounded-lg shadow-2xl border border-zinc-700 flex gap-3 animate-in fade-in zoom-in duration-200">
                            {PROMOTION_PIECES.map(kind => (
                                <button
                                    key={kind}
                                    className="p-2 hover:bg-zinc-700/50 rounded-md flex flex-col items-center justify-center transition-all hover:scale-110 active:scale-95"
                                    onClick={() => handlePromotionSelect(kind)}
                                    title={`Promote to ${kind}`}
                                >
                                    <img
                                        src={getImage(promotionMove.color, kind)}
                                        alt={kind}
                                        className="w-8 h-8 object-cover"
                                    />
                                    <span className="text-xs font-semibold text-white">{kind}</span>
                                </button>
                            ))}
                        </div>
                    </div>
                )}
            </div>

            {/* Bottom Player Card (Self) */}
            <PlayerCard
                display={true}
                color={!flipped ? "White" : "Black"}
                player={
                    !flipped
                        ? (whitePlayer !== undefined ? whitePlayer : board.meta_data.white_player_name)
                        : (blackPlayer !== undefined ? blackPlayer : board.meta_data.black_player_name)
                }
                isTurn={board.turn === (!flipped ? "White" : "Black")}
                rating={
                    !flipped
                        ? board.meta_data.white_player_elo
                        : board.meta_data.black_player_elo
                }
                clock={!flipped ? whiteClock : blackClock} // <- pass appropriate clock
            />


        </div >
    )
}
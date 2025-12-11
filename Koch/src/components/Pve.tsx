import { ChevronDown, Download, Play, RotateCcw, Upload } from "lucide-react";
import { ChessBoard } from "./chessboard/Chessboard";
import { PveLeftPanel } from "./PveLeftPanel";
import { removeDefaultForButton } from "../App";
import { useEffect, useState } from "react";
import { Board } from "../../src-tauri/bindings/Board";
import { GameController } from "../../src-tauri/bindings/GameController";
import { invoke } from "@tauri-apps/api/core";
import { SerializedBoard } from "../../src-tauri/bindings/SerializedBoard";
import { deserialzer, toUciMove } from "../utils";
import { PieceColor } from "../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../src-tauri/bindings/PieceType";

export type GameMode = "pvp" | "pve" | "sandbox"

function gameModeVerbose(gm: GameMode) {
    switch (gm) {
        case "pvp":
            return "Player vs. Player"
        case "pve":
            return "Player vs. Engine"
        case "sandbox":
            return "Sandbox"
        default:
            return "Unknown"
    }
}

export function Pve() {
    const [loading, setLoading] = useState(true);
    const [chessBoard, setChessBoard] = useState<Board | null>(null);
    const [fenPopupOpen, setFenPopupOpen] = useState<boolean>(false);
    const [loadFenInput, setLoadFenInput] = useState<string>("");
    const [gameModeDropdownOpen, setGameModeDropdownOpen] = useState<boolean>(false);
    const [selectedGameMode, setSelectedGameMode] = useState<GameMode>("sandbox");
    const [gameState, setGameState] = useState<GameController | null>(null);
    const [blackTaken, setBlackTaken] = useState<[PieceType, PieceColor][]>([]);
    const [whiteTaken, setWhiteTaken] = useState<[PieceType, PieceColor][]>([]);
    const [moveList, setMoveList] = useState<string[]>([]);
    useEffect(() => {
        (async () => {
            try {
                const board = await invoke<Board>('get_board');
                setChessBoard(board);
            } catch (e) {
                console.error('Failed to get board', e);
            } finally {
                setLoading(false);
            }
        })();
    }, []);
    if (loading) {
        return (
            <div >
                <h1 className="tex-2xl">Loading...</h1>
            </div>

        )
    }
    if (chessBoard === null) {
        return (
            <div>
                <h1 className="tex-2xl">COuld not get board from backend</h1>
            </div>
        )
    }
    async function handleReset() {
        const reset_board = await invoke<Board>('reset_board');
        setChessBoard(reset_board);
    }
    async function handleStartGame() {
        const [board, game] = await invoke<[Board, GameController]>('start_game');
        console.log("Player info:", game.player);
        console.log("Enemy: ", game.enemy);
        setChessBoard(board);
        setGameState(game);
        setTimeout(async () => {
            try {
                const [sb, game] = await invoke<[SerializedBoard, GameController | null]>('update_gameloop');
                const newBoard = deserialzer(sb);

                setChessBoard(newBoard);
                if (game) setGameState(game);

                // Sync UI state (taken pieces, move list) from the authoritative backend board
                setWhiteTaken(newBoard.ui.white_taken);
                setBlackTaken(newBoard.ui.black_taken);
                setMoveList(newBoard.meta_data.move_list.map(m => m.uci));

            } catch (e) {
                console.error("Engine update failed", e);
            }
        }, 500);

    }
    async function handleNewGame() {
        handleStartGame()
    }

    async function handleFenLoad(fenstring: string) {
        const res_board = await invoke<Board>('load_fen', { fen: fenstring })
        setChessBoard(res_board);
    }
    async function handleSandboxBoardMove(from: [number, number], to: [number, number], promotion?: string) {
        try {
            let takenPiece = chessBoard?.squares[to[0]][to[1]];
            const moveResult = await invoke<SerializedBoard | null>('try_move', {
                srcSquare: from,
                destSquare: to,
                promotion: promotion
            });

            if (moveResult) {
                if (takenPiece) {
                    if (takenPiece.color == "Black") {
                        setWhiteTaken(prev => [...prev, [takenPiece.kind, takenPiece.color]]);
                    }
                    if (takenPiece.color == "White") {
                        setBlackTaken(prev => [...prev, [takenPiece.kind, takenPiece.color]]);
                    }
                }
                setMoveList((prev) => [...prev, toUciMove(from, to, promotion)])
                setChessBoard(deserialzer(moveResult));
            } else {
                console.warn('Move was invalid or no result returned');
            }
        } catch (error) {
            console.error('Failed to perform move:', error);
        }
    }

    async function handlePveBoardMove(from: [number, number], to: [number, number]) {
        try {
            // 1. Execute Player Move
            const moveResult = await invoke<SerializedBoard | null>('try_move', {
                srcSquare: from,
                destSquare: to,
            });

            if (moveResult) {
                // Update UI immediately for player
                setChessBoard(deserialzer(moveResult));

                // 2. Trigger Engine Move (async)
                // We use setTimeout to let the React render cycle finish showing the player's move
                // before the engine potentially blocks or delays the next update.
                setTimeout(async () => {
                    try {
                        const [sb, game] = await invoke<[SerializedBoard, GameController | null]>('update_gameloop');
                        const newBoard = deserialzer(sb);

                        setChessBoard(newBoard);
                        if (game) setGameState(game);

                        // Sync UI state (taken pieces, move list) from the authoritative backend board
                        setWhiteTaken(newBoard.ui.white_taken);
                        setBlackTaken(newBoard.ui.black_taken);
                        setMoveList(newBoard.meta_data.move_list.map(m => m.uci));

                    } catch (e) {
                        console.error("Engine update failed", e);
                    }
                }, 500);

            } else {
                console.warn('Move was invalid or no result returned');
            }
        } catch (error) {
            console.error('Failed to perform move:', error);
        }
    }
    return (
        <div className="flex flex-col w-[90%] h-[100%] gap-4">
            {fenPopupOpen && (
                <div className="fixed inset-0 z-50 backdrop-blur-sm bg-black/40 flex justify-center items-center">
                    <div className="bg-dark rounded-xl p-6 shadow-lg min-w-[420px] border-[1px] border-primary">
                        <h2 className="text-secondary mb-[1px] text-lg">Load Position from FEN</h2>
                        <h4 className="text-xs text-secondary/60 mb-4">Enter a FEN string to load a custom position</h4>
                        <label htmlFor="load-fen" className="text-sm text-secondary">Fen String</label>
                        <textarea
                            id="load-fen"
                            className="w-full h-24 text-sm p-2 rounded-lg bg-accent/60 text-secondary placeholder:text-secondary/60 focus:border-accent outline-none resize-none"
                            placeholder={chessBoard.meta_data.starting_position}
                            value={loadFenInput}
                            onChange={(e) => {
                                setLoadFenInput(e.target.value); // removed: setFenPopupOpen(false)
                            }}
                        />
                        <div className="flex flex-col gap-1 mb-2">
                            <span className="text-xs text-secondary/80">
                                FEN (Forsyth-Edwards Notation) is a standard notation for describing chess positions.
                            </span>
                            <span className="text-xs text-secondary/60">
                                The example above is the standard starting position.
                            </span>
                        </div>
                        <div className="flex justify-end gap-2 mt-4">
                            <button
                                className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}
                                onClick={() => setFenPopupOpen(false)}
                            >
                                Cancel
                            </button>
                            <button
                                className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}
                                onClick={(e) => {
                                    e.preventDefault();
                                    handleFenLoad(loadFenInput);
                                    setFenPopupOpen(false); // close after successful load
                                }}
                            >
                                Load
                            </button>
                        </div>
                    </div>
                </div>
            )}
            <div className="flex flex-col w-full h-full">
                <div className="top-bar flex flex-row justify-between items-center bg-primary/40 py-4 px-2">
                    <div className="dropdown relative">
                        <button className={`relative z-30 px-2 w-[160px] h-[40px] rounded transition-colors ${removeDefaultForButton} flex flex-row justify-between items-center gap-1 ${gameModeDropdownOpen ? 'bg-accent' : 'hover:bg-accent'}`}
                            onClick={(e) => { e.preventDefault(); setGameModeDropdownOpen(!gameModeDropdownOpen) }}>
                            <span>{gameModeVerbose(selectedGameMode)}</span>
                            <ChevronDown className="w-4 h-4" />
                        </button>
                        <div className={`z-20 pt-2 absolute top-8 left-0 bg-accent w-[160px] rounded flex flex-col justify-start items-start px-2 overflow-hidden transition-all duration-300 gap-2 pb-2 ${gameModeDropdownOpen ? 'min-h-auto opacity-100' : 'min-h-0 h-0 opacity-0'}`}>
                            <button
                                onClick={(e) => { e.preventDefault(); setSelectedGameMode("sandbox"); setGameModeDropdownOpen(false); }}>
                                <span className="text-md hover:text-secondary">
                                    Sandbox
                                </span>

                            </button>
                            <button
                                onClick={(e) => { e.preventDefault(); setSelectedGameMode("pve"); setGameModeDropdownOpen(false); }}>
                                <span className="text-md hover:text-secondary">
                                    Player vs Engine
                                </span>

                            </button>
                            <button
                                onClick={(e) => { e.preventDefault(); setSelectedGameMode("pvp"); setGameModeDropdownOpen(false); }}>
                                <span className="text-md hover:text-secondary">
                                    Player vs Player
                                </span>
                            </button>
                        </div>
                    </div>
                    <div className="button-group flex felx-row gap-4">
                        <button className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}
                            onClick={(e) => { e.preventDefault(); setFenPopupOpen(true) }}>
                            <Upload className="w-4 h-4 text-amber-50" />
                            <span className="text-amber-50 text-sm">Load FEN</span>

                        </button>
                        <button className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}>
                            <Download className="w-4 h-4 text-amber-50" />
                            <span className="text-amber-50 text-sm">Export</span>
                        </button>
                        {
                            selectedGameMode != "sandbox" ? (
                                gameState && !gameState.game_over ? (
                                    <button className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}
                                        onClick={(e) => { e.preventDefault(); handleNewGame() }}>
                                        <RotateCcw className="w-4 h-4 text-amber-50" />
                                        <span className="text-amber-50 text-sm">Restart Game</span>

                                    </button>
                                ) :
                                    (
                                        <button className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}
                                            onClick={(e) => { e.preventDefault(); handleStartGame() }}>
                                            <Play className="w-4 h-4 text-amber-50" />
                                            <span className="text-amber-50 text-sm">Start Game</span>

                                        </button>
                                    )
                            ) : (

                                <button className={`px-2 py-2 rounded transition-colors hover:bg-accent ${removeDefaultForButton} flex flex-row justify-center items-center gap-1`}
                                    onClick={(e) => { e.preventDefault(); handleReset() }}>
                                    <RotateCcw className="w-4 h-4 text-amber-50" />
                                    <span className="text-amber-50 text-sm">Reset</span>

                                </button>
                            )
                        }
                    </div>
                </div>
                <div className="flex flex-row w-[100%] h-[100%]">
                    <div className="w-full h-full flex justify-center items-center w-[75%]">

                        < ChessBoard board={chessBoard} squareSize={60} onMove={selectedGameMode === "sandbox" ? handleSandboxBoardMove : handlePveBoardMove} flipped={gameState?.player === "Black"} />

                    </div>


                    <PveLeftPanel blackTaken={blackTaken} whiteTaken={whiteTaken} moveList={moveList} />
                </div>

            </div>
        </div >
    )
}
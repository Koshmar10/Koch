import { Ban, ChessKnight, ChevronDown, Clock, Copy, Download, Flag, Flame, Play, RotateCcw, Share, Share2, Upload, Zap } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import { Board } from "../../../src-tauri/bindings/Board";
import { GameController } from "../../../src-tauri/bindings/GameController";
import { SerializedBoard } from "../../../src-tauri/bindings/SerializedBoard";
import { deserialzer, fromUciMove, toUciMove } from "../../utils";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";
import { removeDefaultForButton } from "../../App";
import { ChessBoard } from "../chessboard/Chessboard";
import { PveLeftPanel } from "./PveLeftPanel";
import { squaresAreSame } from "../chessboard/utils";
import { SerializedGameController } from "../../../src-tauri/bindings/SerializedGameController";
import { GameControllerMode } from "../../../src-tauri/bindings/GameControllerMode";
import { TerminationReason } from "../../../src-tauri/bindings/TerminationReason";
import { PveButton } from "./PveButton"
export type GameMode = "pve"

export function formatClock(ms: number | string): string {
    const totalMillis = Number(ms) || 0;
    const minutes = Math.floor((totalMillis / 60000) % 60);
    const seconds = Math.floor((totalMillis / 1000) % 60);
    const millis = totalMillis % 1000;

    const pad = (n: number, len = 2) => n.toString().padStart(len, '0');
    return `${pad(minutes)}:${pad(seconds)}:${pad(millis, 3)}`;
}
export function Pve() {
    const [loading, setLoading] = useState(true);
    const [chessGame, setChessGame] = useState<SerializedGameController | null>(null)
    const [chessBoard, setChessBoard] = useState<Board | null>(null);
    const [lastMove, setLastMove] = useState<[number[], number[]] | null>(null);
    const [gameState, setGameState] = useState<GameController | null>(null);

    const [gameMode, setGameMode] = useState<GameControllerMode>("Rapid")
    const [blackTaken, setBlackTaken] = useState<[PieceType, PieceColor][]>([]);
    const [whiteTaken, setWhiteTaken] = useState<[PieceType, PieceColor][]>([]);
    const [whitePlayerClock, setWhitePlayerClock] = useState<number>(0);
    const [blackPlayerClock, setBlackPlayerClock] = useState<number>(0);
    const [moveList, setMoveList] = useState<string[]>([]);
    const [gameModeDropdownOpen, setGameModeDropdownOpen] = useState<boolean>(false);
    const [selectedGameMode, setSelectedGameMode] = useState<GameControllerMode | null>(null)
    const [startKey, setStartKey] = useState<number>(0)
    const [sharePopupOpen, setSharePopupOpen] = useState<boolean>(false);
    const [shareFen, setShareFen] = useState<string | null>(null);
    const [sharePgn, setSharePgn] = useState<string | null>(null);
    const squareSize = 85;
    const iconSize = 22;
    // 1. Create the ref

    const updateGameMode = async (newMode: GameControllerMode) => {
        setGameMode(newMode);
        let newGame = await invoke<SerializedGameController>('change_gamemode', { newMode });
        setChessGame(newGame);
        setMoveList(newGame.board.meta_data.move_list.split(' '))
        setWhitePlayerClock(newGame.player_clock);
        setBlackPlayerClock(newGame.engine_clock);
        setSelectedGameMode(newGame.mode);
        setChessBoard(deserialzer(newGame.board));
    }
    const startGame = async () => {
        const newGame = await invoke<SerializedGameController>('start_game');
        setChessGame(newGame);
        setWhitePlayerClock(newGame.player_clock);
        setMoveList(newGame.board.meta_data.move_list.split(' '))
        setBlackPlayerClock(newGame.engine_clock);
        setSelectedGameMode(newGame.mode);
        setChessBoard(deserialzer(newGame.board));
        if (newGame.player === "Black") {
            let t = setTimeout(async () => {

                // 1. Execute Player Move
                const newGame = await invoke<SerializedGameController>('update_game_state', { payload: "EngineMove" })
                setChessGame(newGame);
                setWhitePlayerClock(newGame.player_clock);
                setBlackPlayerClock(newGame.engine_clock);
                setSelectedGameMode(newGame.mode);
                setChessBoard(deserialzer(newGame.board));

            }, 500)
            return () => clearTimeout(t);
        }
        setStartKey((prev) => prev + 1);
    };
    const endGame = async (reason: TerminationReason, loser: PieceColor) => {
        try {
            const newGame = await invoke<SerializedGameController>('end_game', { reason, loser });
            setChessGame(newGame);
            setMoveList(newGame.board.meta_data.move_list.split(' '))
            setWhitePlayerClock(newGame.player_clock);
            setBlackPlayerClock(newGame.engine_clock);
            setChessBoard(deserialzer(newGame.board));
        } catch (error) {
            console.error('Failed to end game:', error);
        }
    };
    const resetGame = async () => {
        const newGame = await invoke<SerializedGameController>('new_game');
        setChessGame(newGame)
        setMoveList(newGame.board.meta_data.move_list.split(' '))
        setSelectedGameMode(newGame.mode);
        setWhitePlayerClock(newGame.player_clock);
        setBlackPlayerClock(newGame.engine_clock);
        setChessBoard(deserialzer(newGame.board));
    }
    useEffect(() => {
        // start/stop interval whenever game or board changes
        const interval = setInterval(() => {
            // guard: no game or already ended

            if (!chessGame || chessGame.state === "Ended" || chessGame.state === "AwaitingStart") return;

            if (!chessBoard) return;

            // use functional updates so we always operate on latest values
            if (chessBoard.turn === "Black") {
                setBlackPlayerClock(prev => {
                    const next = Math.max(0, prev - 100);
                    if (next === 0 && chessGame && chessGame.state !== "Ended") {
                        // call endGame once — endGame will update state so subsequent ticks will early return
                        endGame("Timeout", "Black");
                    }
                    return next;
                });
            } else {
                setWhitePlayerClock(prev => {
                    const next = Math.max(0, prev - 100);
                    if (next === 0 && chessGame && chessGame.state !== "Ended") {
                        endGame("Timeout", "White");
                    }
                    return next;
                });
            }
        }, 100);

        return () => clearInterval(interval);
        // include chessGame and chessBoard so interval is recreated/stopped on state change
    }, [chessGame, chessBoard, startKey]);

    useEffect(() => {
        (async () => {
            try {
                const newGame = await invoke<SerializedGameController>('new_game');
                setChessGame(newGame)
                setSelectedGameMode(newGame.mode);
                setWhitePlayerClock(newGame.player_clock);
                setBlackPlayerClock(newGame.engine_clock);
                setChessBoard(deserialzer(newGame.board));
                setMoveList(newGame.board.meta_data.move_list.split(' '))

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


    async function handlePveBoardMove(from: [number, number], to: [number, number], promotion?: string) {
        try {
            // 1. Execute Player Move

            setLastMove([from, to]);
            const newGame = await invoke<SerializedGameController>('update_game_state', { payload: { "Playermove": { from, to, promotion } } })
            setChessGame(newGame);
            setWhitePlayerClock(newGame.player_clock);
            setBlackPlayerClock(newGame.engine_clock);
            setSelectedGameMode(newGame.mode);
            setChessBoard(deserialzer(newGame.board));
            setMoveList(newGame.board.meta_data.move_list.split(' '))
            let t = setTimeout(async () => {

                // 1. Execute Player Move
                const newGame = await invoke<SerializedGameController>('update_game_state', { payload: "EngineMove" })
                setChessGame(newGame);
                setWhitePlayerClock(newGame.player_clock);
                setBlackPlayerClock(newGame.engine_clock);
                setSelectedGameMode(newGame.mode);
                setChessBoard(deserialzer(newGame.board));
                setMoveList(newGame.board.meta_data.move_list.split(' '))

            }, 500)
            return () => clearTimeout(t);


        } catch (error) {
            console.error('Failed to perform move:', error);
        }
    }
    return (
        <div className="flex flex-col w-full h-[100%] gap-4 bg-background-dark">

            <div className="flex flex-col w-full h-full">

                <div className="flex flex-row w-[100%] h-[100%]">
                    <div className="relative w-full h-full flex flex-col justify-center items-center w-[75%]">

                        {/* 3. Attach the ref */}
                        <div className="relative w-auto h-auto">
                            {
                                chessGame && chessGame.state === "Ended" && (
                                    <div className="absolute top-0 left-0 w-full h-full z-40 flex justify-center items-center">
                                        <div
                                            className="backdrop-blur-sm flex justify-center items-center"
                                            style={{ width: squareSize * 8, height: squareSize * 8 }}
                                        >
                                            <div className="bg-card-dark px-12 flex flex-col items-center justify-center rounded-lg shadow-lg p-6 gap-4">

                                                <span className="text-lg font-semibold text-white mb-2">
                                                    {chessGame.termination_reason}
                                                </span>
                                                <span className="text-2xl font-bold text-accent mb-2">
                                                    {chessGame.result}
                                                </span>
                                                <div className="flex flex-row gap-4 mb-2">
                                                    <span className="text-base text-white">745</span>
                                                    <span className="text-base text-white">{chessGame.elo_gain}</span>
                                                </div>
                                                <button className="px-4 py-2 bg-accent text-white rounded hover:bg-accent/80 transition"
                                                    onClick={async () => {
                                                        const res = await invoke<string | null>('save_appgame');

                                                        const newGame = await invoke<SerializedGameController>('new_game');
                                                        setChessGame(newGame)
                                                        setSelectedGameMode(newGame.mode);
                                                        setMoveList(newGame.board.meta_data.move_list.split(' '))
                                                        setWhitePlayerClock(newGame.player_clock);
                                                        setBlackPlayerClock(newGame.engine_clock);
                                                        setChessBoard(deserialzer(newGame.board));

                                                    }}>
                                                    save game
                                                </button>
                                                <button className="px-4 py-2 bg-accent text-white rounded hover:bg-accent/80 transition"
                                                    onClick={() => resetGame()}> reset</button>
                                            </div>
                                        </div>
                                    </div>
                                )
                            }
                            {
                                chessGame && sharePopupOpen && (
                                    <div className="absolute top-0 left-0 w-full h-full z-40 flex justify-center items-center">
                                        <div
                                            className="backdrop-blur-sm flex justify-center items-center"
                                            style={{ width: squareSize * 8, height: squareSize * 8 }}
                                        >
                                            <div className="bg-card-dark p-4 flex flex-col items-center justify-center rounded-xl shadow-lg gap-4 w-[400px]">
                                                <div className="flex flex-col items-center justify-between w-[100%]">
                                                    <div className="flex flex-row w-[100%] pb-3 justify-between mb-4 border-b border-border/60">
                                                        <span className="text-lg font-semibold text-white">Share Game</span>
                                                        <button
                                                            className="text-white hover:text-accent transition text-xl"
                                                            onClick={() => setSharePopupOpen(false)}
                                                            aria-label="Close share popup"
                                                        >
                                                            x
                                                        </button>
                                                    </div>
                                                    <div className="flex flex-col gap-4 w-full">
                                                        <div className="flex flex-col w-full relative">
                                                            <div className="flex flex-row items-end justify-between w-[100%] mb-2 ">

                                                                <label className="text-xs text-muted mb-1">FEN</label>
                                                                <button
                                                                    className=" text-xs text-foreground-dark rounded hover:text-accent transition pr-2"
                                                                    onClick={() => {
                                                                        if (shareFen) {
                                                                            navigator.clipboard.writeText(shareFen);
                                                                        }
                                                                    }}
                                                                    aria-label="Copy Fen"
                                                                    type="button"
                                                                >
                                                                    <Copy size={16} />
                                                                </button>
                                                            </div>
                                                            <textarea
                                                                className="w-full bg-primary/15 text-foreground-dark rounded px-2 py-1 text-sm resize-none"
                                                                value={shareFen || ""}
                                                                readOnly
                                                                rows={2}
                                                            />

                                                        </div>
                                                        <div className="flex flex-col w-full relative">
                                                            <div className="flex flex-row items-end justify-between w-[100%] mb-2 ">

                                                                <label className="text-xs text-muted mb-1">PGN</label>
                                                                <button
                                                                    className=" text-xs text-foreground-dark rounded hover:text-accent transition pr-2"
                                                                    onClick={() => {
                                                                        if (sharePgn) {
                                                                            navigator.clipboard.writeText(sharePgn);
                                                                        }
                                                                    }}
                                                                    aria-label="Copy PGN"
                                                                    type="button"
                                                                >
                                                                    <Copy size={16} />
                                                                </button>
                                                            </div>
                                                            <textarea
                                                                className="w-full bg-primary/15 text-foreground-dark rounded px-2 py-1 text-sm resize-none "
                                                                value={sharePgn || ""}
                                                                readOnly
                                                                rows={16}
                                                            />

                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                )

                            }
                            <ChessBoard
                                board={chessBoard}
                                squareSize={squareSize}
                                onMove={handlePveBoardMove}
                                flipped={chessGame ? chessGame.player === "Black" : undefined}
                                lastMove={lastMove}
                                //isInteractive={chessGame ? chessGame.state === "Ongoing" : undefined}
                                user={chessGame ? chessGame.player : undefined}
                                whiteClock={formatClock(whitePlayerClock)}
                                blackClock={formatClock(blackPlayerClock)}
                                whitePlayer={chessGame?.player_card}
                                blackPlayer={chessGame?.engine_card}

                            />
                        </div>
                        <div className="flex flex-row gap-4 items-center mt-4">

                            <button
                                onClick={() => { if (!chessGame) return; endGame("Resignation", chessGame.player === "Black" ? "Black" : "White") }}

                            >


                                <PveButton icon={chessGame?.can_be_abandoned ? <Ban className="text-destructive-dark" size={iconSize} /> : <Flag className="text-destructive-dark" size={iconSize} />} tooltip={chessGame?.can_be_abandoned ? 'Abbandon' : 'Surrender'} disabled={chessGame?.state === "AwaitingStart"} />
                            </button>
                            <button

                                onClick={async () => {
                                    if (chessGame?.state !== "Ongoing") {
                                        startGame();
                                    }
                                }}
                            >
                                <PveButton icon={chessGame?.state === "Ended" ? <RotateCcw size={iconSize} /> : <Play size={iconSize} />} tooltip="Start game" disabled={chessGame?.state === "Ongoing"} />

                            </button>

                            <div className="relative flex items-center justify-center">
                                <button
                                    onClick={() => setGameModeDropdownOpen((open) => !open)}
                                >
                                    <PveButton
                                        icon={
                                            selectedGameMode === "Classical" ? (
                                                <ChessKnight size={iconSize} /> // Replace with <ChessKnight /> if you have that icon
                                            ) : selectedGameMode === "Rapid" ? (
                                                <Clock size={iconSize} />
                                            ) : selectedGameMode === "Blitz" ? (
                                                <Zap size={iconSize} /> // Replace with <Zap /> if you have that icon
                                            ) : selectedGameMode === "Bullet" ? (
                                                <Flame size={iconSize} /> // Replace with <Flame /> if you have that icon
                                            ) : (
                                                <Clock size={iconSize} />
                                            )
                                        }
                                        tooltip={!gameModeDropdownOpen ? "Change game mode" : null}
                                        disabled={chessGame?.state === "Ongoing"}
                                    />
                                </button>
                                {gameModeDropdownOpen && (
                                    <div
                                        className="absolute flex flex-col shadow-md rounded-md mb-2 z-10 min-w-[240px] bottom-full left-1/2 -translate-x-1/2 bg-background-dark"
                                    >
                                        {/*
                                          Use same base styling as the Bullet option for all entries.
                                          Show mode icon + label on the left, time on the right.
                                        */}
                                        <button
                                            className={`px-4 py-2 hover:bg-primary text-left rounded-t-md flex justify-between items-center text-foreground-dark ${selectedGameMode === "Bullet" ? "bg-primary font-bold" : "bg-primary/40"}`}
                                            onClick={() => { setGameModeDropdownOpen(false); updateGameMode("Bullet"); }}
                                        >
                                            <div className="flex items-center gap-3">
                                                <Flame size={iconSize} className="text-primary-dark" />
                                                <span className="capitalize">bullet</span>
                                            </div>
                                            <span>1 min</span>
                                        </button>

                                        <button
                                            className={`px-4 py-2 hover:bg-primary text-left flex justify-between items-center text-foreground-dark ${selectedGameMode === "Blitz" ? "bg-primary font-bold" : "bg-primary/40"}`}
                                            onClick={() => { setGameModeDropdownOpen(false); updateGameMode("Blitz"); }}
                                        >
                                            <div className="flex items-center gap-3">
                                                <Zap size={iconSize} className="text-primary-dark" />
                                                <span className="capitalize">blitz</span>
                                            </div>
                                            <span>3 min</span>
                                        </button>

                                        <button
                                            className={`px-4 py-2 hover:bg-primary text-left flex justify-between items-center text-foreground-dark ${selectedGameMode === "Rapid" ? "bg-primary font-bold" : "bg-primary/40"}`}
                                            onClick={() => { setGameModeDropdownOpen(false); updateGameMode("Rapid"); }}
                                        >
                                            <div className="flex items-center gap-3">
                                                <Clock size={iconSize} className="text-primary-dark" />
                                                <span className="capitalize">rapid</span>
                                            </div>
                                            <span>10 min</span>
                                        </button>

                                        <button
                                            className={`px-4 py-2 hover:bg-primary text-left rounded-b-md flex justify-between items-center text-foreground-dark ${selectedGameMode === "Classical" ? "bg-primary font-bold" : "bg-primary/40"}`}
                                            onClick={() => { setGameModeDropdownOpen(false); updateGameMode("Classical"); }}
                                        >
                                            <div className="flex items-center gap-3">
                                                <ChessKnight size={iconSize} className="text-primary-dark" />
                                                <span className="capitalize">classical</span>
                                            </div>
                                            <span>30 min</span>
                                        </button>
                                    </div>
                                )}
                            </div>



                            <button onClick={async () => {
                                setSharePopupOpen(true)
                                const [fen, pgn] = await invoke<[string, string]>('get_share_data');
                                setShareFen(fen);
                                setSharePgn(pgn);

                            }} className="focus:outline-none">
                                <PveButton icon={<Share2 size={iconSize} />} tooltip="Share game" />
                            </button>
                        </div>
                    </div>


                    <PveLeftPanel blackTaken={blackTaken} whiteTaken={whiteTaken} moveList={moveList} opening={chessBoard.meta_data.opening || undefined} />
                </div>

            </div>
        </div >
    )
}
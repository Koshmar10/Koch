import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Dispatch, SetStateAction, useEffect, useRef, useState } from "react";

import { Bot, ChevronDown, ChevronFirst, ChevronLast, ChevronLeft, ChevronRight, ChevronUp, Download, FileText, Flame, GitBranch, Layers, Lightbulb, LoaderCircle, MessageSquareText, Target, Upload } from "lucide-react";

import { mockMessages } from "../mock";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { AnalyzerController } from "../../../src-tauri/bindings/AnalyzerController";
import type { PvObject } from "../../../src-tauri/bindings/PvObject";
import type { PvLineData } from "../../../src-tauri/bindings/PvLineData";

import { EvalType } from "../../../src-tauri/bindings/EvalType";
import { ArrowData } from "../ArrowLayer";
import { EngineData } from "./EngineData";
import { ChessBoard } from "../chessboard/Chessboard";
import { AnalyzerInputs } from "./AnalyzerInputs";
import { ChatBox } from "./ChatBox";
import { SerializedAnalyzerController } from "../../../src-tauri/bindings/SerializedAnalyzerController";
import { deserializeAnalyzerController, fromUciMove } from "../../utils";
import { Board } from "../../../src-tauri/bindings/Board";
import { AnalyzerSettings } from "./AnalyzrerSettings";
import { NotificationManager } from "../notifications/NotificationManager";
import { GameSelectPopup } from "./GameSelectPopup";


interface AnalyzerProps {
    gameId?: number | null;
    setSelectedGameId?: Dispatch<SetStateAction<number | null>>;
}

export function Analyzer({ gameId, setSelectedGameId }: AnalyzerProps) {
    const [analyzerContoller, setAnalyzerController] = useState<AnalyzerController | null>(null);
    const [toggleChat, setToggleChat] = useState<boolean>(true);
    const [barEval, setBarEval] = useState<number>(0.0);
    const [evalKind, _setEvalKind] = useState<EvalType>("Centipawn");
    const [playerColor, _setPlayerColor] = useState<PieceColor>("White");
    const [currentMove, setCurrentMove] = useState<number>(-1);
    const [currentMoveUci, setCurrentMoveUci] = useState<string>("");
    const [pvObject, setPvObject] = useState<PvObject | null>(null);
    const [engineRunning, setEngineRunning] = useState<boolean>(true);
    const [engineLoading, setEngineLoading] = useState<boolean>(false);
    const [gameSelectPopupOpen, setGameSelectPopupOpen] = useState<boolean>(true);
    // Helper: update UCI from index safely
    const setUciForIndex = (index: number, ctrl: AnalyzerController | null) => {
        const len = ctrl?.board.meta_data.move_list.length ?? 0;
        if (index >= 0 && index < len) {
            setCurrentMoveUci(ctrl!.board.meta_data.move_list[index].uci);
        } else {
            setCurrentMoveUci("");
        }
    };
    const fen = useRef<string>("");
    const [analyzerMode, setAnalyzerMode] = useState<"sandbox" | "game">("sandbox");
    const squareSize = 72;

    const lastFenRef = useRef<string>("");

    // Race control refs
    const latestIndexRef = useRef<number>(-1);
    const latestTokenRef = useRef<number>(0);

    const [displayBoard, setDisplayBoard] = useState<Board | null>(null);

    useEffect(() => {
        async function load() {
            if (gameId == null) {
                const ac = await invoke<AnalyzerController>("fetch_default_game");
                setAnalyzerController(ac);
                setAnalyzerMode("sandbox");
                fen.current = ac.board.meta_data.starting_position;
                lastFenRef.current = fen.current; // set FEN for engine
                const idx = ac.current_ply ?? -1;
                setCurrentMove(idx);
                latestIndexRef.current = idx;
                setDisplayBoard(ac.board);
                setUciForIndex(idx, ac);
                // send eval signal on load
                try {
                    await invoke('set_analyzer_fen', { currentMove: -1 });
                } catch (e) {
                    console.error('set_analyzer_fen on load failed', e);
                }
            } else {
                const ac = await invoke<AnalyzerController>("fetch_game", { id: gameId });
                setAnalyzerController(ac);
                setAnalyzerMode("game");
                fen.current = ac.board.meta_data.starting_position;
                lastFenRef.current = fen.current; // set FEN for engine
                const idx = ac.current_ply ?? -1;
                setCurrentMove(idx);
                latestIndexRef.current = idx;
                setDisplayBoard(ac.board);
                setUciForIndex(idx, ac);
                // send eval signal on load
                try {
                    await invoke('set_analyzer_fen', { currentMove: latestIndexRef.current });
                } catch (e) {
                    console.error('set_analyzer_fen on load failed', e);
                }
            }
        }
        load();
    }, [gameId])

    async function startEngine() {
        if (!analyzerContoller) return;
        try {
            await invoke('set_analyzer_fen', { currentMove: latestIndexRef.current });
            setEngineRunning(true);
        } catch (e) {
            console.error("Failed to start engine:", e);
        }
    }

    const fetchIndex = async (index: number, token: number) => {
        try {
            const data = await invoke<SerializedAnalyzerController | null>("get_board_at_index", { moveIndex: index });
            if (!data) return;
            if (latestTokenRef.current !== token) return;

            lastFenRef.current = data.serialized_board.fen;
            const newController = deserializeAnalyzerController(data);
            setAnalyzerController(newController);
            setDisplayBoard(newController.board);
            setUciForIndex(index, newController); // update UCI for the fetched index

            // Push eval immediately after fetch
            if (engineRunning) {
                await invoke('set_analyzer_fen', { currentMove: index });
            }
        } catch (e) {
            console.error("get_board_at_index failed", e);
        }
    };

    const openGameSelectPopup = () => {
        setGameSelectPopupOpen(true);
    }

    const handleMoveStep = async (direction: number) => {
        if (!analyzerContoller) return;

        const maxIndex = analyzerContoller.board.meta_data.move_list.length;
        let newIndex = currentMove + direction;

        // Clamp: -1 = start, 0..maxIndex-1
        if (newIndex < -1) newIndex = -1;
        if (newIndex >= maxIndex) newIndex = maxIndex - 1;
        if (newIndex === currentMove) return;

        setCurrentMove(newIndex);
        latestIndexRef.current = newIndex;
        setUciForIndex(newIndex, analyzerContoller);

        // New token for this request
        const token = latestTokenRef.current + 1;
        latestTokenRef.current = token;

        // Fetch immediately
        await fetchIndex(newIndex, token);
    };

    const handleFirstMove = async () => {
        if (!analyzerContoller) return;
        const newIndex = -1;
        if (currentMove === newIndex) return;

        setCurrentMove(newIndex);
        latestIndexRef.current = newIndex;
        setUciForIndex(newIndex, analyzerContoller);

        const token = latestTokenRef.current + 1;
        latestTokenRef.current = token;

        await fetchIndex(newIndex, token);
    };

    const handleLastMove = async () => {
        if (!analyzerContoller) return;
        const lastIndex = analyzerContoller.board.meta_data.move_list.length - 1;
        if (currentMove === lastIndex) return;

        setCurrentMove(lastIndex);
        latestIndexRef.current = lastIndex;
        setUciForIndex(lastIndex, analyzerContoller);

        const token = latestTokenRef.current + 1;
        latestTokenRef.current = token;

        await fetchIndex(lastIndex, token);
    };
    //onMove?: (from: [number, number], to: [number, number], promotion?: string) => void;
    const handleOnMove = async (from: [number, number], to: [number, number], promotion?: string) => {
        if (analyzerMode !== "sandbox") return;
        console.log(`Board Moved from ${from} to ${to}`)
        try {
            const serializedAnalyzer = await invoke<SerializedAnalyzerController | null>('try_analyzer_move', {
                srcSquare: from,
                destSquare: to,
                promotion: promotion
            });
            if (serializedAnalyzer) {
                const deserializedAnalyzer = deserializeAnalyzerController(serializedAnalyzer);
                setAnalyzerController(deserializedAnalyzer);
                setDisplayBoard(deserializedAnalyzer.board);
                try {
                    await invoke('set_analyzer_fen', { currentMove: latestIndexRef.current });
                } catch (e) {
                    console.error('set_analyzer_fen on load failed', e);
                }
            }
        }
        catch (e) { console.log(e); }

    }
    // Listen once for PV updates
    useEffect(() => {
        let unlisten: (() => void) | undefined;
        (async () => {
            try {
                unlisten = await listen<PvObject>("pv_update", (e) => setPvObject(e.payload));
            } catch (e) {
                console.error("pv_update listen failed", e);
            }
        })();
        return () => { try { unlisten && unlisten(); } catch { } };
    }, []);

    // Helper to get first available PV line
    const getFirstPvLine = (pv: PvObject | null): PvLineData | null => {
        if (!pv || !pv.lines) return null;
        const keys = Object.keys(pv.lines).map(k => Number(k)).sort((a, b) => a - b);
        for (const k of keys) {
            const line = pv.lines[k];
            if (line) return line;
        }
        return null;
    };

    return (


        <div className="main-section w-full flex flex-row flex-1 overflow-hidden">
            {/* <NotificationManager /> */}

            <AnalyzerSettings analyzer={analyzerContoller} setEngineLoading={setEngineLoading} startEngine={startEngine} openGameSelectPopup={openGameSelectPopup} />
            <div className="w-full flex flex-col relative">
                <GameSelectPopup isRendered={gameSelectPopupOpen} closePopup={setGameSelectPopupOpen} setSelectedGameId={setSelectedGameId} />
                <div className="flex flex-row-reverse text-secondary h-20 w-full px-6 items-center">
                    <div className="h-fit w-fit bg-primary/30 rounded-lg py-3 px-6 flex flex-row gap-3 items-center">
                        <button
                            className="transition-colors hover:bg-primary/50 rounded-full p-1 border border-transparent p-2"
                            title="Download"
                            onClick={() => { /* TODO: download action */ }}
                        >
                            <Upload size={18} />
                        </button>
                        <button
                            className="transition-colors hover:bg-primary/50 rounded-full p-1 border border-transparent p-2"
                            title="Upload"
                            onClick={() => { /* TODO: upload action */ }}
                        >
                            <Download size={18} />
                        </button>
                        <button
                            className="transition-colors hover:bg-primary/50 rounded-full p-1 border border-transparent p-2"
                            title="Document"
                            onClick={() => { /* TODO: document action */ }}
                        >
                            <FileText size={18} />
                        </button>
                        <button
                            className={`transition-colors hover:bg-primary/50 rounded-full p-1 border border-transparent p-2 ${!toggleChat ? "bg-red-900/30" : ""}`}
                            title="Chat"
                            onClick={(e) => {
                                e.preventDefault();
                                setToggleChat(!toggleChat);
                            }}
                        >
                            <MessageSquareText size={18} />
                        </button>
                    </div>
                </div>
                <div className="board-section flex flex-1 justify-center items-center">
                    <div className={`eval-bar w-14 h-[48%] bg-black/40 border border-accent/60 mr-8 flex ${playerColor == "White" ? 'flex-col' : 'flex-col-reverse'}`}>

                        <div className="black-eval h-full w-full
                        flex items-start justify-center">

                        </div>
                        <div
                            className="white-eval bg-secondary w-full flex justify-center items-end transition-[height] duration-150 ease-out"
                            style={{
                                height: (() => {
                                    const line = getFirstPvLine(pvObject);
                                    if (!line) return `${50 + Math.max(-5, Math.min(5, barEval)) * 10}%`; // fallback to barEval (pawns)
                                    // line.eval_kind: "Centipawn" | "Mate", line.eval_value: number (cp or mate distance)
                                    if (line.eval_kind === "Mate") {
                                        return line.eval_value < 0 ? "0%" : "100%";
                                    }
                                    const pawns = (line.eval_value ?? 0) / 100; // cp -> pawns
                                    const pct = 50 + Math.max(-5, Math.min(5, pawns)) * 10;
                                    const clamped = Math.max(0, Math.min(100, pct));
                                    return `${clamped}%`;
                                })(),
                            }}
                        >
                            <span className="eval-text text-dark mb-2 text-xs font-bold">
                                {(() => {
                                    const line = getFirstPvLine(pvObject);
                                    if (!line) return barEval > 0 ? `+${barEval.toFixed(2)}` : barEval.toFixed(2);
                                    if (line.eval_kind === "Mate") {
                                        const v = line.eval_value ?? 0;
                                        const sign = v > 0 ? "+" : v < 0 ? "-" : "";
                                        return `${sign}M${Math.abs(v)}`;
                                    }
                                    const pawns = (line.eval_value ?? 0) / 100;
                                    return pawns > 0 ? `+${pawns.toFixed(2)}` : pawns.toFixed(2);
                                })()}
                            </span>
                        </div>

                    </div>
                    <div className="relative">
                        {displayBoard && ( // RENDER FROM displayBoard
                            <ChessBoard board={displayBoard} squareSize={squareSize} lastMove={fromUciMove(currentMoveUci)} onMove={handleOnMove} />
                        )}
                    </div>
                    <AnalyzerInputs
                        height={squareSize * 8}
                        currentMove={currentMove + 1}
                        totalMoves={analyzerContoller?.board.meta_data.move_list.length || 0}
                        onPrev={() => handleMoveStep(-1)}
                        onNext={() => handleMoveStep(1)}
                        onFirst={handleFirstMove}
                        onLast={handleLastMove}
                        setEngineLoading={setEngineLoading}
                        engineLoading={engineLoading}
                        engineRunning={engineRunning}
                        setEngineRunning={setEngineRunning}
                        pvObject={pvObject}           // pass down
                        startEngine={startEngine}      // pass down
                    />
                </div>
            </div>
            <ChatBox toggleChat={toggleChat} />
        </div >

    );
}
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { Bot, ChevronFirst, ChevronLast, ChevronLeft, ChevronRight, Flame, GitBranch, Layers, Lightbulb, LoaderCircle, MessageSquareText, Target } from "lucide-react";

import { mockMessages } from "../mock";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { AnalyzerController } from "../../../src-tauri/bindings/AnalyzerController";
import { PvObject } from "../../../src-tauri/bindings/PvObject";

import { EvalType } from "../../../src-tauri/bindings/EvalType";
import { ArrowData } from "../ArrowLayer";
import { EngineData } from "./EngineData";
import { ChessBoard } from "../chessboard/Chessboard";
import { AnalyzerInputs } from "./AnalyzerInputs";
import { ChatBox } from "./ChatBox";
import { SerializedAnalyzerController } from "../../../src-tauri/bindings/SerializedAnalyzerController";
import { deserializeAnalyzerController, fromUciMove } from "../../utils";
import { Board } from "../../../src-tauri/bindings/Board";


interface AnalyzerProps {
    gameId?: number | null;
}

export function Analyzer({ gameId }: AnalyzerProps) {
    const [analyzerContoller, setAnalyzerController] = useState<AnalyzerController | null>(null);
    const [toggleChat, setToggleChat] = useState<boolean>(true);
    const [barEval, setBarEval] = useState<number>(0.0);
    const [evalKind, _setEvalKind] = useState<EvalType>("Centipawn");
    const [playerColor, _setPlayerColor] = useState<PieceColor>("White");
    const [currentMove, setCurrentMove] = useState<number>(-1);
    const [currentMoveUci, setCurrentMoveUci] = useState<string>("");
    const [pvObject, setPvObject] = useState<PvObject | null>(null);
    const [engineRunning, setEngineRunning] = useState<boolean>(true);


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
    const [arrows, setArrows] = useState<ArrowData[]>([])
    const [analyzerMode, setAnalyzerMode] = useState<"sandbox" | "game">("sandbox");
    const squareSize = 60;

    const timerRef = useRef<number | null>(null);
    const lastFenRef = useRef<string>("");

    // Race control & debounce (fără stări suplimentare)
    const lastClickTsRef = useRef<number>(0);
    const latestIndexRef = useRef<number>(-1);
    const latestTokenRef = useRef<number>(0);
    const debounceTimerRef = useRef<number | null>(null);
    const settleTimerRef = useRef<number | null>(null);
    const rapidThresholdMs = 180;
    const settleDelayMs = 220;

    const handleFenChange = (newFen: string) => {
        fen.current = newFen;

        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }

        if (!engineRunning) return;

        timerRef.current = window.setTimeout(async () => {
            try {
                await invoke('set_analyzer_fen', { fen: fen.current });
            } catch (e) {
                console.error('set_analyzer_fen failed', e);
            } finally {
                timerRef.current = null;
            }
        }, 400);
    };

    const [displayBoard, setDisplayBoard] = useState<Board | null>(null); // REINTRODUCED

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
                setDisplayBoard(ac.board);
                setUciForIndex(idx, ac);
                // send eval signal on load
                try {
                    await invoke('set_analyzer_fen', { fen: lastFenRef.current });
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
                setDisplayBoard(ac.board);
                setUciForIndex(idx, ac);
                // send eval signal on load
                try {
                    await invoke('set_analyzer_fen', { fen: lastFenRef.current });
                } catch (e) {
                    console.error('set_analyzer_fen on load failed', e);
                }
            }
        }
        load();
        return () => {
            if (timerRef.current) clearTimeout(timerRef.current);
            if (debounceTimerRef.current) clearTimeout(debounceTimerRef.current);
            if (settleTimerRef.current) clearTimeout(settleTimerRef.current);
        };
    }, [gameId])
    
    async function startEngine() {
        if (!analyzerContoller) return;
        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
        try {
            await invoke('set_analyzer_fen', { fen: fen.current });
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
        } catch (e) {
            console.error("get_board_at_index failed", e);
        }
    };

    const pushFenToEngine = async () => {
        try {
            if (engineRunning && lastFenRef.current) {
                await invoke('set_analyzer_fen', { fen: lastFenRef.current });
            }
        } catch (e) {
            console.error('set_analyzer_fen failed', e);
        }
    };

    const handleMoveStep = async (direction: number) => {
        if (!analyzerContoller) return;

        const now = performance.now();
        const delta = now - lastClickTsRef.current;
        lastClickTsRef.current = now;
        const isRapid = delta > 0 && delta < rapidThresholdMs;

        const maxIndex = analyzerContoller.board.meta_data.move_list.length;
        let newIndex = currentMove + direction;

        // -1 = poziția de start, 0..maxIndex-1 = după N mutări
        if (newIndex < -1) newIndex = -1;
        if (newIndex >= maxIndex) newIndex = maxIndex - 1;
        if (newIndex === currentMove) return;

        setCurrentMove(newIndex);
        latestIndexRef.current = newIndex;
        setUciForIndex(newIndex, analyzerContoller); // update immediately for highlight

        // Anulează debounce/settle anterioare
        if (debounceTimerRef.current) {
            clearTimeout(debounceTimerRef.current);
            debounceTimerRef.current = null;
        }
        if (settleTimerRef.current) {
            clearTimeout(settleTimerRef.current);
            settleTimerRef.current = null;
        }

        // Token nou pentru această secvență
        const token = latestTokenRef.current + 1;
        latestTokenRef.current = token;

        if (isRapid) {
            // Debounce pentru clicuri rapide
            debounceTimerRef.current = window.setTimeout(() => {
                fetchIndex(latestIndexRef.current, latestTokenRef.current);
            }, rapidThresholdMs);

            // Fetch final când utilizatorul se oprește brusc
            settleTimerRef.current = window.setTimeout(async () => {
                if (latestTokenRef.current === token) {
                    await fetchIndex(latestIndexRef.current, latestTokenRef.current);
                    await pushFenToEngine();
                }
            }, settleDelayMs);
        } else {
            // Clic “liniștit”: fetch imediat + sincronizare motor
            await fetchIndex(newIndex, token);
            await pushFenToEngine();
        }
    };

    const handleFirstMove = async () => {
        const newIndex = -1;
        if (!analyzerContoller || currentMove === newIndex) return;

        setCurrentMove(newIndex);
        latestIndexRef.current = newIndex;
        setUciForIndex(newIndex, analyzerContoller); // clear UCI at start (-1)

        if (debounceTimerRef.current) { clearTimeout(debounceTimerRef.current); debounceTimerRef.current = null; }
        if (settleTimerRef.current) { clearTimeout(settleTimerRef.current); settleTimerRef.current = null; }

        const token = latestTokenRef.current + 1;
        latestTokenRef.current = token;

        await fetchIndex(newIndex, token);
        await pushFenToEngine();
    };

    const handleLastMove = async () => {
        if (!analyzerContoller) return;
        const lastIndex = analyzerContoller.board.meta_data.move_list.length - 1;
        if (currentMove === lastIndex) return;

        setCurrentMove(lastIndex);
        latestIndexRef.current = lastIndex;
        setUciForIndex(lastIndex, analyzerContoller);

        if (debounceTimerRef.current) { clearTimeout(debounceTimerRef.current); debounceTimerRef.current = null; }
        if (settleTimerRef.current) { clearTimeout(settleTimerRef.current); settleTimerRef.current = null; }

        const token = latestTokenRef.current + 1;
        latestTokenRef.current = token;

        await fetchIndex(lastIndex, token);
        await pushFenToEngine();
    };

    return (
        <div className="flex flex-col w-full h-full">
            <div className="flex flex-row justify-between bg-primary/80 text-secondary pt-4 px-4 pb-4 h-14">
                <h1>Analyzer Top Bar</h1>
                <button
                    className="hover:text-secondary/90"
                    onClick={(e) => {
                        e.preventDefault();
                        setToggleChat(!toggleChat);
                    }}
                >
                    <MessageSquareText />
                </button>
            </div>
            <div className="main-section w-full flex flex-row flex-1 overflow-hidden">
                <div className="flex flex-col  h-full w-[18%] min-w-[200px]">
                    <div className="px-4 py-2 bg-primary/20 border-b-2 border-r-2 border-accent/60 h-auto">
                        <div className="flex items-center gap-2 mb-3">
                            <Layers className="h-4 w-4 text-foreground/50" />
                            <span className="text-xs uppercase tracking-wider text-foreground/60">
                                Analyzer Mode
                            </span>
                        </div>
                        <div className="flex flex-row gap-4 mb-4">
                            <button onClick={(e) => [
                                e.preventDefault()
                            ]}>Sandbox</button>
                            <button>LoadGame</button>
                        </div>
                        <div>
                            {analyzerMode == "sandbox" ? (
                                <span>Sandbox mode </span>
                            ) : (

                                analyzerContoller && (
                                    <div className="flex flex-col gap-1 text-sm">

                                        <span>{analyzerContoller.board.meta_data.date}</span>
                                        <span>{analyzerContoller.board.meta_data.opening}</span>
                                        <span>{analyzerContoller.board.meta_data.termination} {analyzerContoller.board.meta_data.result}</span>
                                        <span>
                                            {`${analyzerContoller.board.meta_data.white_player_name} (${analyzerContoller.board.meta_data.white_player_elo}) vs ${analyzerContoller.board.meta_data.black_player_name} (${analyzerContoller.board.meta_data.black_player_elo})`}
                                        </span>
                                    </div>
                                )

                            )}
                        </div>
                    </div>
                    <div className="px-4 py-2 bg-primary/20 border-r-2 border-accent/60 h-full">

                        <div className="h-full flex flex-col overflow-y-auto">
                            <div className="flex items-center gap-2 mb-2">
                                <GitBranch className="h-4 w-4 text-secondary/60" />
                                <span className="text-xs uppercase tracking-wider text-secondary/60">Move Timeline</span>
                            </div>
                            {analyzerContoller && analyzerContoller.board.meta_data.move_list.map((mv, idx) => {
                                const isActive = currentMove === idx + 1;
                                return (
                                    <button
                                        key={idx}
                                        className={`flex items-center gap-3 px-2 py-1 text-xs w-full text-left
                                            ${isActive ? 'text-secondary font-semibold' : 'text-secondary/70 hover:text-secondary'}
                                        `}

                                    >
                                        <span className="inline-flex h-5 w-5 items-center justify-center text-secondary/50 text-xs">
                                            {idx + 1}.
                                        </span>
                                        <span className={`${isActive ? 'bg-accent/20 px-1 rounded' : ''} truncate text-sm`}>
                                            {mv.uci}
                                        </span>
                                    </button>
                                );
                            })}
                        </div>
                    </div>
                </div>
                <div className="board-section flex flex-1 justify-center items-center">
                    <div className={`eval-bar w-14 h-[48%] bg-black/40 border border-accent/60 mr-8 flex   ${playerColor == "White" ? 'flex-col' : 'flex-col-reverse'}`}>

                        <div className="black-eval h-full w-full
                        flex items-start justify-center">

                        </div>
                        <div
                            className="white-eval bg-secondary w-full flex justify-center items-end transform transition duration-300 ease-out"
                            style={{
                                height:
                                    (() => {
                                        const evalObj = analyzerContoller?.board.meta_data.move_list[currentMove]?.evaluation;
                                        const isMate = evalKind === "Mate" || evalObj?.kind === "Mate";
                                        if (isMate) return evalObj && evalObj.value < 0 ? "10%" : "900%";
                                        return `calc(${100 + barEval * 10}% )`;
                                    })(),
                            }}
                        >
                            <span className="eval-text text-dark mb-2 text-xs font-bold">
                                {(() => {
                                    const evalObj = analyzerContoller?.board.meta_data.move_list[currentMove]?.evaluation;
                                    const isMate = evalKind === "Mate" || evalObj?.kind === "Mate";
                                    if (isMate) {
                                        const mateVal = evalObj?.value ?? 0;
                                        const sign = mateVal > 0 ? "+" : mateVal < 0 ? "-" : "";
                                        return `${sign}M${Math.abs(mateVal)}`;
                                    }
                                    return barEval > 0 ? `+${barEval.toFixed(2)}` : barEval.toFixed(2);
                                })()}
                            </span>
                        </div>

                    </div>
                    <div className="relative">
                        {displayBoard && ( // RENDER FROM displayBoard
                            <ChessBoard board={displayBoard} squareSize={squareSize} lastMove={fromUciMove(currentMoveUci)} />
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
                        engineRunning={engineRunning}
                        setEngineRunning={setEngineRunning}
                        pvObject={pvObject}
                        startEngine={startEngine}
                    />
                </div>
                <ChatBox toggleChat={toggleChat} />
            </div >
        </div >
    )
}
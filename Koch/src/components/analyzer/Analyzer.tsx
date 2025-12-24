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
import { ChessBoard, GhostPiece, RenderedPiece } from "../chessboard/Chessboard";
import { AnalyzerInputs } from "./AnalyzerInputs";
import { ChatBox } from "./ChatBox";
import { SerializedAnalyzerController } from "../../../src-tauri/bindings/SerializedAnalyzerController";
import { deserializeAnalyzerController, fileRankToRowCol, fromUciMove } from "../../utils";
import { Board } from "../../../src-tauri/bindings/Board";
import { AnalyzerSettings } from "./AnalyzrerSettings";
import { NotificationManager } from "../notifications/NotificationManager";
import { GameSelectPopup } from "./GameSelectPopup";

import { AiChatMessage } from "../../../src-tauri/bindings/AiChatMessage";
import { AiChatMessageRole } from "../../../src-tauri/bindings/AiChatMessageRole";
import { LocalMessage } from "../../../src-tauri/bindings/LocalMessage";

export type UiAiChatMessage = {
    id: number,
    role: AiChatMessageRole,
    text: string,
    sent_at: string,
    move_index: number,
    allow_hover: boolean,
}
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
    const [gameSelectPopupOpen, setGameSelectPopupOpen] = useState<boolean>(false);
    const [whiteClock, setWhiteClock] = useState<string | null>(null);
    const [blackClock, setBlackClock] = useState<string | null>(null);
    const [suggestion, setSuggestion] = useState<ArrowData | null>(null);
    // Active toggles moved to parent-level state
    const [threatsActive, setThreatsActive] = useState<boolean>(false);
    const [suggestionsActive, setSuggestionsActive] = useState<boolean>(false);
    const [attacksActive, setAttacksActive] = useState<boolean>(false);
    const [isFetchig, setIsFetching] = useState<boolean>(false);

    const fen = useRef<string>("");
    const [analyzerMode, setAnalyzerMode] = useState<"sandbox" | "game">("sandbox");
    const squareSize = 72;

    const lastFenRef = useRef<string>("");

    // Race control refs
    const latestIndexRef = useRef<number>(-1);
    const latestTokenRef = useRef<number>(0);

    const [displayBoard, setDisplayBoard] = useState<Board | null>(null);
    // separate state for threat so fetching it doesn't interfere with board/fetch flows
    const [threat, setThreat] = useState<ArrowData | null>(null);
    const latestThreatToken = useRef(0);
    const [hoverTarget, setHoverTarget] = useState<[number, number] | null>(null);

    const handleOnSquareHover = (sq: string) => { setHoverTarget(fileRankToRowCol(sq)); }
    const handleOnSquareHoverLost = () => { setHoverTarget(null); }

    const [chatHistory, setChatHistory] = useState<UiAiChatMessage[]>([]);
    const [aiThinking, setAiThinking] = useState<boolean>(false);
    const [ghostPieces, setGhostPieces] = useState<GhostPiece[]>([]);
    const [ghostArrows, setGhostArrows] = useState<ArrowData[]>(
        []
    )
    const handleOnMoveHover = (move: string) => {
        let from_sq = `${move[0]}${move[1]}`;
        let to_sq = `${move[2]}${move[3]}`;
        let from: [number, number] = fileRankToRowCol(from_sq);
        let to: [number, number] = fileRankToRowCol(to_sq);
        let piece = analyzerContoller?.board.squares[from[0]][from[1]];
        if (!piece) return;
        setGhostPieces([{ kind: piece ? piece.kind : "King", color: piece ? piece.color : 'Black', r: to[0], c: to[1], }]);

        setGhostArrows([{ from: `${from[0]}-${from[1]}`, to: `${to[0]}-${to[1]}`, color: "lime", type: "ghost" }])
    }
    const handleOnMoveHoveLost = () => { setGhostPieces([]); setGhostArrows([]); }
    const chatMsgId = useRef(0);
    const chatUpdateKey = useRef(0);
    const updateChatHistory = (role: "System" | "User" | "Assistant" | "Function" | "Tool", msg: string) => {
        setChatHistory((prev) => {
            const now = new Date();
            const formattedTime = now.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
            return [
                ...prev,
                {
                    id: chatMsgId.current,
                    role: role,
                    text: msg,
                    sent_at: formattedTime,
                    move_index: analyzerContoller ? analyzerContoller.current_ply : -1,
                    allow_hover: true,
                } as UiAiChatMessage,
            ];
        });
        chatMsgId.current += 1;
        chatUpdateKey.current += 1;
    }
    useEffect(() => {
        if (chatHistory.length === 0) return;
        let lastMsg = chatHistory[chatHistory.length - 1];
        if (lastMsg.role === 'User') {
            const thinkingTimer = window.setTimeout(() => {
                setAiThinking(true);
            }, 500);
            const timer = window.setTimeout(async () => {
                console.log("now a request to api is sent");
                try {
                    const llmResponse = await invoke<[string, number] | null>('send_llm_request', { msg: lastMsg.text });
                    const llmMsg = llmResponse ? llmResponse[0] : "";
                    const mv_index = llmResponse ? llmResponse[1] : -1;
                    updateChatHistory("System", llmMsg);
                } catch (e) {
                    console.error("send_llm_request failed", -1);
                    updateChatHistory("System", "");
                } finally {
                    setAiThinking(false);
                }
            }, 500);
            return () => { window.clearTimeout(timer); window.clearTimeout(thinkingTimer) };

        }
    }, [chatHistory])
    // Helper: update UCI from index safely
    const setUciForIndex = (index: number, ctrl: AnalyzerController | null) => {
        const len = ctrl?.board.meta_data.move_list.length ?? 0;
        if (index >= 0 && index < len) {
            setCurrentMoveUci(ctrl!.board.meta_data.move_list[index].uci);
        } else {
            setCurrentMoveUci("");
        }
    };
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
                if (ac.chat_history) {
                    setChatHistory(
                        ac.chat_history.chat_messages.map((aiMsg: LocalMessage, idx: number) => {
                            // Format the ISO date string to a user-friendly time (e.g., "12:13 PM")
                            let formattedTime = "";
                            try {
                                const date = new Date(aiMsg.sent_at);
                                formattedTime = date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
                            } catch {
                                formattedTime = aiMsg.sent_at;
                            }
                            return {
                                id: idx,
                                role: aiMsg.role,
                                text: aiMsg.content,
                                sent_at: formattedTime,
                                move_index: aiMsg.move_index,
                                allow_hover: aiMsg.move_index == ac.current_ply,

                            } as UiAiChatMessage;
                        })
                    );
                }
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
        if (isFetchig) return;
        try {
            setSuggestion(null);
            setThreat(null);
            setIsFetching(true);
            const data = await invoke<SerializedAnalyzerController | null>("get_board_at_index", { moveIndex: index });
            if (!data) return;
            if (latestTokenRef.current !== token) return;
            lastFenRef.current = data.serialized_board.fen;
            const newController = deserializeAnalyzerController(data);
            setAnalyzerController(newController);
            setDisplayBoard(newController.board);
            setCurrentMove(newController.current_ply);
            setUciForIndex(index, newController); // update UCI for the fetched index
            setChatHistory((prevChatHistory) =>
                prevChatHistory.map((msg) => ({
                    ...msg,
                    allow_hover: msg.move_index === newController.current_ply, // Allow hover only if move_index matches current move
                }))
            );
            //console.log(newController.board.move_cache);
            // kick analyzer eval / threat as fire-and-forget (do not block UI)

            try {
                if (engineRunning) {
                    debouncedSetAnalyzerFen(index, token); // Use the debounced function
                }
                // threat can be emitted by backend analyzer thread via `threat_update` event
            } catch (e) {
                console.error("post-fetch analyzer trigger failed", e);
            }
        } catch (e) {
            console.error("get_board_at_index failed", e);
        } finally {
            setIsFetching(false);
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
                debouncedSetAnalyzerFen(latestIndexRef.current, latestTokenRef.current); // Use the debounced function
            }
        } catch (e) {
            console.error(e);
        }

    }
    // Listen once for PV updates
    useEffect(() => {
        let unlisten: (() => void) | undefined;
        (async () => {
            try {
                unlisten = await listen<PvObject>("pv_update", (e) => {
                    setPvObject(e.payload)
                    updateSuggestion(e.payload)
                });
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
    const updateSuggestion = (pv: PvObject | null) => {
        try {
            if (!pv || !pv.lines) {
                setSuggestion(null);
                return;
            }

            // pick the best line by eval_value (fall back to first available)
            const keys = Object.keys(pv.lines).map(k => Number(k)).sort((a, b) => a - b);
            let bestLine: PvLineData | null = null;
            let bestVal = Number.NEGATIVE_INFINITY;
            for (const k of keys) {
                const line = pv.lines[k];
                if (!line) continue;
                const v = typeof line.eval_value === "number" ? line.eval_value : 0;
                if (v > bestVal) {
                    bestVal = v;
                    bestLine = line;
                }
            }
            if (!bestLine) {
                bestLine = getFirstPvLine(pv);
            }
            if (!bestLine || !bestLine.moves) {
                setSuggestion(null);
                return;
            }

            // first token in the PV moves string should be a uci-like move "e2e4"
            const firstToken = (bestLine.moves || "").split(/\s+/)[0];
            if (!firstToken || firstToken.length < 4) {
                setSuggestion(null);
                return;
            }
            const fromSq = firstToken.slice(0, 2);
            const toSq = firstToken.slice(2, 4);
            // validate coordinates are a..h and 1..8
            const sqRe = /^[a-h][1-8]$/;
            if (!sqRe.test(fromSq) || !sqRe.test(toSq)) {
                console.warn("updateSuggestion: invalid PV token, skipping", firstToken, bestLine.moves);
                setSuggestion(null);
                return;
            }
            // convert to row/col indices and set suggestion as ArrowData
            let from: number[], to: number[];
            try {
                from = fileRankToRowCol(fromSq);
                to = fileRankToRowCol(toSq);
            } catch (err) {
                console.error("updateSuggestion: fileRankToRowCol failed", err, fromSq, toSq);
                setSuggestion(null);
                return;
            }
            setSuggestion({
                from: `${from[0]}-${from[1]}`,
                to: `${to[0]}-${to[1]}`,
                color: "blue",
                type: "engine",
            });
        } catch (err) {
            console.error("updateSuggestion failed:", err);
            setSuggestion(null);
        }
    }

    // Parse a raw threat string (e.g. "e2e4 ...") and set threat as ArrowData if valid.
    const updateThreat = (raw: string | null) => {
        try {
            if (!raw) {
                setThreat(null);
                return;
            }
            const trimmed = raw.trim();
            if (!trimmed) {
                setThreat(null);
                return;
            }

            const firstToken = trimmed.split(/\s+/)[0];
            if (!firstToken || firstToken.length < 4) {
                // not a UCI-like token -> ignore for arrow overlay
                setThreat(null);
                return;
            }
            const fromSq = firstToken.slice(0, 2);
            const toSq = firstToken.slice(2, 4);
            const sqRe = /^[a-h][1-8]$/;
            if (!sqRe.test(fromSq) || !sqRe.test(toSq)) {
                console.warn("updateThreat: invalid token, skipping", firstToken);
                setThreat(null);
                return;
            }
            let from: number[], to: number[];
            try {
                from = fileRankToRowCol(fromSq);
                to = fileRankToRowCol(toSq);
            } catch (err) {
                console.error("updateThreat: fileRankToRowCol failed", err, fromSq, toSq);
                setThreat(null);
                return;
            }
            setThreat({
                from: `${from[0]}-${from[1]}`,
                to: `${to[0]}-${to[1]}`,
                color: "red",
                type: "engine",
            });
        } catch (err) {
            console.error("updateThreat failed:", err);
            setThreat(null);
        }
    }
    // Stop engine on component unload (unmount)
    useEffect(() => {
        return () => {
            try {
                invoke('stop_analyzer');
            } catch (e) {
                console.error('Failed to stop analyzer on unload', e);
            }
        };
    }, []);

    // When the current move index changes, pick the matching clock token (if present).
    // If index is even -> set whiteClock to clocks[index], clear blackClock.
    // If index is odd  -> set blackClock to clocks[index], clear whiteClock.
    useEffect(() => {
        if (!analyzerContoller) return;
        if (gameId !== -1) return;
        if (currentMove < 0) {
            setWhiteClock(null);
            setBlackClock(null);
            return;
        }
        let clock = analyzerContoller.board.meta_data.move_list[currentMove].clock;
        if (currentMove < analyzerContoller.board.meta_data.move_list.length) {
            if (currentMove % 2 === 0) {
                setWhiteClock(clock);

            } else {
                setBlackClock(clock);
            }
        } else {
            setWhiteClock(null);
            setBlackClock(null);
        }
        //console.log(whiteClock, blackClock);
    }, [currentMove, analyzerContoller]);

    // trigger without awaiting on critical path
    useEffect(() => {
        let unlisten: (() => void) | undefined;
        (async () => {
            try {
                unlisten = await listen<string>("threat_update", (e) => {
                    //console.log("threat_update:", e.payload);
                    updateThreat(e.payload);
                });
            } catch (err) { console.error("threat_update listen failed", err); }
        })();
        return () => { try { unlisten && unlisten(); } catch { } };
    }, []);
    const debounceTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const debouncedSetAnalyzerFen = (currentMove: number, token: number) => {
        if (debounceTimeoutRef.current) {
            clearTimeout(debounceTimeoutRef.current); // Clear the previous timeout
        }

        debounceTimeoutRef.current = setTimeout(async () => {
            try {
                await invoke('set_analyzer_fen', { currentMove });
            } catch (e) {
                console.error('set_analyzer_fen failed', e);
            }
            // Only call get_threat if not at the end move
            if (analyzerContoller && currentMove < analyzerContoller.board.meta_data.move_list.length - 1) {
                const t = await invoke<string | null>('get_threat');
                // only apply threat if this fetch token still current (avoid races)
                if (latestTokenRef.current === token) {
                    updateThreat(t);
                }
            }
        }, 200); // 500ms debounce delay
    };
    useEffect(() => {
        return () => {
            if (debounceTimeoutRef.current) {
                clearTimeout(debounceTimeoutRef.current);
            }
        };
    }, []);
    return (


        <div className="main-section w-full h-full flex flex-row flex-1 overflow-hidden">
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
                    <div className={`eval-bar w-14 bg-black/40 border border-accent/60 mr-8 flex ${playerColor == "White" ? 'flex-col' : 'flex-col-reverse'}`
                    } style={{ height: squareSize * 8 }}>

                        <div className="black-eval h-full w-full
                        flex items-start justify-center">
                        </div>
                        <div
                            className="white-eval bg-secondary w-full flex justify-center items-end transition-[height] duration-150 ease-out"
                            style={{
                                height: (() => {
                                    const barMax = 8 * squareSize;
                                    const line = getFirstPvLine(pvObject);
                                    if (!line) {
                                        // fallback to barEval (pawns)
                                        const pawns = Math.max(-5, Math.min(5, barEval));
                                        const pct = 0.5 + pawns / 10; // -5..5 -> 0..1
                                        return `${Math.round(barMax * pct)}px`;
                                    }
                                    if (line.eval_kind === "Mate") {
                                        return line.eval_value < 0 ? "0px" : `${barMax}px`;
                                    }
                                    const pawns = Math.max(-5, Math.min(5, (line.eval_value ?? 0) / 100));
                                    const pct = 0.5 + pawns / 10; // -5..5 -> 0..1
                                    const px = Math.round(barMax * pct);
                                    return `${Math.max(0, Math.min(barMax, px))}px`;
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
                            <ChessBoard
                                isInteractive={analyzerMode === "sandbox" ? true : false}
                                board={displayBoard}
                                squareSize={squareSize}
                                lastMove={fromUciMove(currentMoveUci)}
                                onMove={handleOnMove}
                                whiteClock={whiteClock ?? undefined}
                                blackClock={blackClock ?? undefined}
                                suggestion={suggestionsActive ? suggestion : null}
                                threat={threatsActive ? threat : null}
                                tintEnabled={attacksActive}
                                hoverTarget={hoverTarget}
                                ghostPieces={ghostPieces}
                                ghostArrows={ghostArrows}
                            />
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
                        // pass active state values & setters down
                        threatsActive={threatsActive}
                        setThreatsActive={setThreatsActive}
                        suggestionsActive={suggestionsActive}
                        setSuggestionsActive={setSuggestionsActive}
                        attacksActive={attacksActive}
                        setAttacksActive={setAttacksActive}
                        isFetching={isFetchig}
                    />
                </div>
            </div>
            <ChatBox
                toggleChat={toggleChat}
                chatHistory={chatHistory}
                updateChatHistory={updateChatHistory}
                thinking={aiThinking}
                handleOnSquareHover={handleOnSquareHover}
                handleOnSquareHoverLost={handleOnSquareHoverLost}
                fetchIndex={(index: number) => fetchIndex(index, latestTokenRef.current)} // Pass fetchIndex as a prop
                handleOnMoveHover={handleOnMoveHover}
                handleOnMoveHoverLost={handleOnMoveHoveLost}
            />
        </div >

    );
}
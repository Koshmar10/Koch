import { invoke } from "@tauri-apps/api/core";
import { use, useEffect, useMemo, useState } from "react";
import { Board } from "../../src-tauri/bindings/Board";
import { Bot, ChevronFirst, ChevronLast, ChevronLeft, ChevronRight, Flame, Lightbulb, MessageSquareText, Target } from "lucide-react";
import { AnalyzerBoard } from "./AnalyzerBoard";
import { ChessPiece } from "../../src-tauri/bindings/ChessPiece";
import { mockMessages } from "./mock";
import { PieceColor } from "../../src-tauri/bindings/PieceColor";
import { BoardMetaData } from "../../src-tauri/bindings/BoardMetaData";
import { AnalyzerController } from "../../src-tauri/bindings/AnalyzerController";


const mock2Moves: string[] = [
    "e2e4",
    "e7e5",
    "g1f3",
    "b8c6",
    "f1c4",
    "g8f6",
    "d2d3",
    "f8c5",
    "c2c3",
    "d7d6",
    "b1d2",
    "b8c6",
    "f1c4",
    "g8f6",
    "d2d3",
    "f8c5",
    "c2c3",
    "d7d6",
    "b1d2",
    "c8g4"
];
interface AnalyzerProps {
    gameId?: number | null;
}

export function Analyzer({ gameId }: AnalyzerProps) {
    const [analyzerContoller, setAnalyzerController] = useState<AnalyzerController | null>(null);
    const [toggleChat, setToggleChat] = useState<boolean>(true);
    const [loading, setLoading] = useState(true);
    const [barEval, setBarEval] = useState<number>(0.0);
    const [playerColor, setPlayerColor] = useState<PieceColor>("White");
    const [currentMove, setCurrentMove] = useState<number>(0);
    const [threatsActive, setThreatsActive] = useState<boolean>(false);
    const [suggestionsActive, setSuggestionActive] = useState<boolean>(false);
    const [attacksActive, setAttacksActive] = useState<boolean>(false);
    const squareSize = 60;
    useEffect(() => {
        async function load() {
            if (gameId == null) return;
            const ac = await invoke<AnalyzerController>("fetch_game", { id: gameId });
            setAnalyzerController(ac);
        }
        load();
    }, [gameId])
    if (!analyzerContoller) {
        return (
            <div className="flex justify-center items-center w-full h-full">
                <h1 className="text-xl">
                    No Anayzer Opened
                </h1>
            </div>
        )
    }
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
                <div className="board-section flex flex-1 justify-center items-center">
                    <div className={`eval-bar w-14 h-[48%] bg-black/40 border border-accent/60 mr-8 flex  ${playerColor == "White" ? 'flex-col' : 'flex-col-reverse'}`}>
                        <div className="black-eval h-[50%] w-full
                        flex items-start justify-center">
                            <span className="eval-text text-secondary mt-2 text-xs font-bold">
                                -0.33
                            </span>
                        </div>
                        <div className="white-eval bg-secondary h-[50%] w-full flex justify-center items-end">
                            <span className="eval-text text-dark mb-2 text-xs font-bold">
                                +0.33
                            </span>
                        </div>
                    </div>
                    <div className="relative">
                        <div
                            className=" absolute top-[-3.5rem] left-0 bg-black flex flex-row justify-center gap-2 px-4 py-2 border-2 border-accent/70 rounded-md"
                            style={{ maxWidth: `calc(${squareSize}px * 8)` }}
                        >
                            <div className="flex flex-row gap-2 overflow-x-scroll w-[98%] text-secondary ">

                                {
                                    analyzerContoller?.board.meta_data.move_list.map((mv, idx) => (
                                        <span className={`${idx == currentMove ? 'bg-primary/90' : 'bg-primary/50'} px-2 rounded-sm`}>
                                            {mv.uci}
                                        </span>
                                    ))
                                }
                            </div>
                        </div>
                        {
                            analyzerContoller &&
                            <AnalyzerBoard squareSize={squareSize} board={analyzerContoller.board} playerColor={playerColor} />
                        }
                    </div>
                    <div className="analyzer-inputs flex flex-col justify-start items-start h-[50%] gap-4 pt-2 mx-4  w-full max-w-[160px]">
                        <div className="flex flex-col w-full gap-2">
                            <p className="text-secondary/50 text-xs">Navigation</p>
                            <div className="flex flex-row justify-center w-full gap-2">
                                <button className="w-full flex justify-center items-center bg-primary/20 border-[1px]  border-accent/70 rounded-sm py-1">
                                    <ChevronFirst className="text-secondary/50 w-4 h-4" />
                                </button>
                                <button className="w-full flex justify-center items-center bg-primary/20 border-[1px]  border-accent/70 rounded-sm"><ChevronLast className="text-secondary/50 w-4 h-4" /></button>
                            </div>
                            <div className="flex flex-row justify-center w-full gap-2">
                                <button className="w-full flex justify-center items-center bg-primary/20 border-[1px] border-accent/70 rounded-sm py-1"
                                    disabled={currentMove == 0}
                                    onClick={async (e) => {
                                        e.preventDefault();
                                        if (!analyzerContoller) return;

                                        // Undo the last applied move
                                        const idx = currentMove - 1;
                                        const uci = analyzerContoller.board.meta_data.move_list[idx]?.uci;
                                        if (!uci) return;

                                        const analyzer = await invoke<AnalyzerController>('undo_move', { uci });
                                        setAnalyzerController(analyzer);
                                        setCurrentMove(Math.max(0, currentMove - 1));
                                    }}>
                                    <ChevronLeft className="text-secondary/90 w-4 h-4" />
                                </button>
                                <button className="w-full flex justify-center items-center bg-primary/20 border-[1px] border-accent/70 rounded-sm py-1"
                                    disabled={!!analyzerContoller?.board && currentMove >= (analyzerContoller?.board.meta_data.move_list.length || 0)}
                                    onClick={async (e) => {
                                        e.preventDefault();
                                        if (!analyzerContoller) return;

                                        // Apply the next move
                                        const idx = currentMove;
                                        const uci = analyzerContoller.board.meta_data.move_list[idx]?.uci;
                                        if (!uci) return;

                                        const analyzer = await invoke<AnalyzerController>('do_move', { uci });
                                        setAnalyzerController(analyzer);
                                        setCurrentMove(Math.min(analyzerContoller.board.meta_data.move_list.length, currentMove + 1));
                                    }}>
                                    <ChevronRight className="text-secondary/90 w-4 h-4" />
                                </button>
                            </div>
                            <button className="my-2  py-2 px-3 bg-accent text-md text-secondary/80">
                                Move {currentMove}
                            </button>
                        </div>

                        <div className="flex flex-col w-full gap-2">
                            <p className="text-secondary/50 text-xs" >Analysis Tools</p>
                            <div className={`flex flex-row w-full justify-start items-center gap-2 text-sm  p-1 rounded-md hover:cursor-pointer ${threatsActive ? 'bg-primary/60' : 'hover:bg-primary/60'}`}
                                onClick={(e) => {
                                    e.preventDefault();
                                    setThreatsActive(!threatsActive);
                                }}>
                                <Flame className="w-4 h-4 text-secondary"
                                />
                                <span
                                    className="text-secondary">Threats</span>
                            </div>
                            <div
                                className={`flex flex-row w-full justify-start items-center gap-2  p-1 rounded-md text-sm
                                hover:cursor-pointer 
                                ${attacksActive ? 'bg-primary/60' : 'hover:bg-primary/60'}`}
                                onClick={(e) => {
                                    e.preventDefault();
                                    setAttacksActive(!attacksActive);
                                }}>
                                <Target
                                    className="w-4 h-4 text-secondary" />
                                <span
                                    className="text-secondary">Attacks</span>
                            </div>
                            <div
                                className={`flex flex-row w-full justify-start items-center gap-2
                                 p-1 rounded-md text-sm
                                hover:cursor-pointer
                                ${suggestionsActive ? 'bg-primary/60' : ' hover:bg-primary/60 '}`}
                                onClick={(e) => {
                                    e.preventDefault();
                                    setSuggestionActive(!suggestionsActive);
                                }}>
                                <Lightbulb className="w-4 h-4 text-secondary " />
                                <span className="text-secondary">Suggestion</span>
                            </div>
                        </div>

                    </div>
                </div>

                <div
                    className={`chat-box flex flex-col bg-primary/20 border-l border-accent/80 transition-all duration-300 ease-in-out ${toggleChat ? 'w-0' : 'w-[30%]'
                        }`}
                >
                    {/* HEADER */}
                    <div className="flex flex-row items-center gap-3 px-4 h-14 shrink-0">
                        <div className="text-white/20 bg-primary/80 p-1 rounded-full flex justify-center items-center">
                            <Bot className="w-9 h-8" />
                        </div>
                        <div className="flex flex-col">
                            <span className="text-md">Koch AI</span>
                            <span className="text-xs text-secondary/90">Online</span>
                        </div>
                    </div>

                    {/* MESSAGES (flex-grow, scrollable) */}
                    <div className="flex-1 flex flex-col overflow-y-auto border-y border-accent/80 px-3 py-2 space-y-4">
                        {mockMessages.map((msg, idx) => (
                            <div
                                key={idx}
                                className={`${msg.sender === "User"
                                    ? 'self-end bg-primary/50'
                                    : 'self-start bg-accent/50'
                                    } w-[60%] flex flex-col p-3 rounded-lg gap-2`}
                            >
                                <div
                                    className={`flex ${msg.sender === "User"
                                        ? 'flex-row-reverse'
                                        : 'flex-row'
                                        } justify-between text-xs`}
                                >
                                    <span >{msg.sender}</span>
                                    <span >{msg.time}</span>
                                </div>
                                <div className="text-sm leading-snug text-secondary/80 font-sans">
                                    {msg.text}
                                </div>
                            </div>
                        ))}
                    </div>

                    {/* INPUT (fixed height, stays visible) */}
                    <div className="flex flex-row items-center gap-2 px-3 h-14 shrink-0">
                        <input
                            type="text"
                            className="flex-1 bg-primary/60 text-secondary text-sm px-3 py-2 rounded outline-none focus:ring-1 focus:ring-accent placeholder:text-secondary/60"
                            placeholder="Type message..."
                        />
                        <button className="px-4 py-2 bg-accent/80 rounded text-sm hover:bg-accent transition-colors">
                            Send
                        </button>
                    </div>
                </div>
            </div >
        </div >
    )
}
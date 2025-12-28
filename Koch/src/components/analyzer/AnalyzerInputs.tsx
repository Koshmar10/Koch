import { ChevronFirst, ChevronLast, ChevronLeft, ChevronRight, Flame, Lightbulb, Target } from "lucide-react";
import { Dispatch, SetStateAction, useState } from "react";
import { EngineData } from "./EngineData";
import { PvObject } from "../../../src-tauri/bindings/PvObject";

interface AnalyzerInputsProps {
    height: number;
    // Navigation State
    currentMove: number;
    totalMoves: number;
    // Navigation Actions
    onPrev: (step: number) => void;
    onNext: (step: number) => void;
    onFirst: () => void;
    onLast: () => void;
    // Engine Data Props (Pass-through)
    engineRunning: boolean;
    engineLoading: boolean;
    setEngineLoading: Dispatch<SetStateAction<boolean>>;
    setEngineRunning: Dispatch<SetStateAction<boolean>>;
    pvObject: PvObject | null;

    startEngine: () => void;

    // NEW: active toggles moved to parent
    threatsActive: boolean;
    setThreatsActive: Dispatch<SetStateAction<boolean>>;
    suggestionsActive: boolean;
    setSuggestionsActive: Dispatch<SetStateAction<boolean>>;
    attacksActive: boolean;
    setAttacksActive: Dispatch<SetStateAction<boolean>>;
    isFetching: boolean;
}

export function AnalyzerInputs({
    height,
    currentMove,
    totalMoves,
    onPrev,
    onNext,
    onFirst,
    onLast,
    engineRunning,
    engineLoading,
    setEngineRunning,
    pvObject,
    startEngine,
    threatsActive,
    setThreatsActive,
    suggestionsActive,
    setSuggestionsActive,
    attacksActive,
    setAttacksActive,
    isFetching,
}: AnalyzerInputsProps) {

    return (
        <div className="analyzer-inputs flex flex-col justify-start items-start gap-4 mx-4 w-full max-w-[160px]"
            style={{ height: `${height}px` }}>

            <div className="flex flex-col w-full gap-2">
                <p className="text-foreground-dark/70 text-xs">Navigation</p>
                <div className="flex flex-row justify-center w-full gap-2">
                    {/* First */}
                    <button
                        className="w-full flex justify-center items-center bg-card-dark border-[1px] border-accent/50 rounded-sm py-1 disabled:opacity-50 text-foreground-dark/80 hover:text-primary/50"
                        disabled={currentMove === -1 || isFetching}
                        onClick={(e) => { e.preventDefault(); onFirst(); }}
                    >
                        <ChevronFirst size={20} />
                    </button>

                    {/* Prev */}
                    <button
                        className="w-full flex justify-center items-center bg-card-dark border-[1px] border-accent/50 rounded-sm py-1 disabled:opacity-50 text-foreground-dark/80 hover:text-primary/50"
                        disabled={currentMove === -1 || isFetching}
                        onClick={(e) => { e.preventDefault(); onPrev(-1); }}
                    >
                        <ChevronLeft size={20} />
                    </button>

                    {/* Next */}
                    <button
                        className="w-full flex justify-center items-center bg-card-dark border-[1px] border-accent/50 rounded-sm py-1 disabled:opacity-50 text-foreground-dark/80 hover:text-primary/50"
                        disabled={currentMove >= totalMoves || isFetching}
                        onClick={(e) => { e.preventDefault(); onNext(1); }}
                    >
                        <ChevronRight size={20} />
                    </button>

                    {/* Last */}
                    <button
                        className="w-full flex justify-center items-center bg-card-dark border-[1px] border-accent/50 rounded-sm py-1 disabled:opacity-50 text-foreground-dark/80 hover:text-primary/50"
                        disabled={currentMove >= totalMoves || isFetching}
                        onClick={(e) => { e.preventDefault(); onLast(); }}
                    >
                        <ChevronLast size={20} />
                    </button>
                </div>

                <button className="my-2 py-2 px-3 bg-primary/90 text-md text-foreground-dark/90">
                    Move {currentMove}
                </button>
            </div>

            <div className="flex flex-col w-full gap-2">
                <p className="text-foreground-dark/70 text-xs">Analysis Tools</p>
                {/* ... Tools UI remains the same ... */}
                <div className={`flex flex-row w-full justify-start items-center gap-2 text-sm p-1 rounded-md hover:cursor-pointer ${threatsActive ? 'bg-primary/60' : 'hover:bg-primary/60'}`}
                    onClick={(e) => { e.preventDefault(); setThreatsActive(v => !v); }}>
                    <Flame className="w-4 h-4 text-foreground-dark" />
                    <span className="text-foreground-dark">Threats</span>
                </div>
                <div className={`flex flex-row w-full justify-start items-center gap-2 p-1 rounded-md text-sm hover:cursor-pointer ${attacksActive ? 'bg-primary/60' : 'hover:bg-primary/60'}`}
                    onClick={(e) => { e.preventDefault(); setAttacksActive(v => !v); }}>
                    <Target className="w-4 h-4 text-foreground-dark" />
                    <span className="text-foreground-dark">Attacks</span>
                </div>
                <div className={`flex flex-row w-full justify-start items-center gap-2 p-1 rounded-md text-sm hover:cursor-pointer ${suggestionsActive ? 'bg-primary/60' : ' hover:bg-primary/60 '}`}
                    onClick={(e) => { e.preventDefault(); setSuggestionsActive(v => !v); }}>
                    <Lightbulb className="w-4 h-4 text-foreground-dark " />
                    <span className="text-foreground-dark">Suggestion</span>
                </div>
            </div>

            <EngineData
                engineRunning={engineRunning}
                setEngineRunning={setEngineRunning}
                engineLoading={engineLoading}
                pvObject={pvObject}
                startEngine={startEngine}
            />
        </div>
    )
}
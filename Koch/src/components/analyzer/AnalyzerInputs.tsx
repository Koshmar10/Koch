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
    setEngineRunning: Dispatch<SetStateAction<boolean>>;
    pvObject: PvObject | null;
    startEngine: () => void;
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
    setEngineRunning,
    pvObject,
    startEngine
}: AnalyzerInputsProps) {

    const [threatsActive, setThreatsActive] = useState<boolean>(false);
    const [suggestionsActive, setSuggestionActive] = useState<boolean>(false);
    const [attacksActive, setAttacksActive] = useState<boolean>(false);

    return (
        <div className="analyzer-inputs flex flex-col justify-start items-start gap-4 mx-4 w-full max-w-[160px]"
            style={{ height: `${height}px` }}>

            <div className="flex flex-col w-full gap-2">
                <p className="text-secondary/50 text-xs">Navigation</p>
                <div className="flex flex-row justify-center w-full gap-2">
                    {/* First */}
                    <button
                        className="w-full flex justify-center items-center bg-primary/20 border-[1px] border-accent/70 rounded-sm py-1 disabled:opacity-50"
                        disabled={currentMove === -1}
                        onClick={(e) => { e.preventDefault(); onFirst(); }}
                    >
                        <ChevronFirst className="text-secondary/50 w-4 h-4" />
                    </button>

                    {/* Prev */}
                    <button
                        className="w-full flex justify-center items-center bg-primary/20 border-[1px] border-accent/70 rounded-sm py-1 disabled:opacity-50"
                        disabled={currentMove === -1}
                        onClick={(e) => { e.preventDefault(); onPrev(-1); }}
                    >
                        <ChevronLeft className="text-secondary/90 w-4 h-4" />
                    </button>

                    {/* Next */}
                    <button
                        className="w-full flex justify-center items-center bg-primary/20 border-[1px] border-accent/70 rounded-sm py-1 disabled:opacity-50"
                        disabled={currentMove >= totalMoves}
                        onClick={(e) => { e.preventDefault(); onNext(1); }}
                    >
                        <ChevronRight className="text-secondary/90 w-4 h-4" />
                    </button>

                    {/* Last */}
                    <button
                        className="w-full flex justify-center items-center bg-primary/20 border-[1px] border-accent/70 rounded-sm disabled:opacity-50"
                        disabled={currentMove >= totalMoves}
                        onClick={(e) => { e.preventDefault(); onLast(); }}
                    >
                        <ChevronLast className="text-secondary/50 w-4 h-4" />
                    </button>
                </div>

                <button className="my-2 py-2 px-3 bg-accent text-md text-secondary/80">
                    Move {currentMove}
                </button>
            </div>

            <div className="flex flex-col w-full gap-2">
                <p className="text-secondary/50 text-xs">Analysis Tools</p>
                {/* ... Tools UI remains the same ... */}
                <div className={`flex flex-row w-full justify-start items-center gap-2 text-sm p-1 rounded-md hover:cursor-pointer ${threatsActive ? 'bg-primary/60' : 'hover:bg-primary/60'}`}
                    onClick={(e) => { e.preventDefault(); setThreatsActive(!threatsActive); }}>
                    <Flame className="w-4 h-4 text-secondary" />
                    <span className="text-secondary">Threats</span>
                </div>
                <div className={`flex flex-row w-full justify-start items-center gap-2 p-1 rounded-md text-sm hover:cursor-pointer ${attacksActive ? 'bg-primary/60' : 'hover:bg-primary/60'}`}
                    onClick={(e) => { e.preventDefault(); setAttacksActive(!attacksActive); }}>
                    <Target className="w-4 h-4 text-secondary" />
                    <span className="text-secondary">Attacks</span>
                </div>
                <div className={`flex flex-row w-full justify-start items-center gap-2 p-1 rounded-md text-sm hover:cursor-pointer ${suggestionsActive ? 'bg-primary/60' : ' hover:bg-primary/60 '}`}
                    onClick={(e) => { e.preventDefault(); setSuggestionActive(!suggestionsActive); }}>
                    <Lightbulb className="w-4 h-4 text-secondary " />
                    <span className="text-secondary">Suggestion</span>
                </div>
            </div>

            <EngineData
                engineRunning={engineRunning}
                setEngineRunning={setEngineRunning}
                pvObject={pvObject}
                startEngine={startEngine}
            />
        </div>
    )
}
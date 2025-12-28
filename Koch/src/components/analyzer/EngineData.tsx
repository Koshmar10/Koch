import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event"; // Import listen
import { PvObject } from "../../../src-tauri/bindings/PvObject";
import { LoaderCircle } from "lucide-react";
import { Dispatch, SetStateAction, useEffect, useState } from "react"; // Import useEffect, useState

interface EngineDataProps {
    engineRunning: boolean;
    setEngineRunning: Dispatch<SetStateAction<boolean>>
    engineLoading: boolean
    pvObject: PvObject | null; // Initial/Parent state
    startEngine: () => void;
}

export function EngineData({ pvObject, engineRunning, setEngineRunning, startEngine, engineLoading }: EngineDataProps) {
    // Local state to ho    ld the live up
    const [localPv, setLocalPv] = useState<PvObject | null>(pvObject);

    useEffect(() => {
        setLocalPv(pvObject);
    }, [pvObject]);


    return (
        <div className="analyzer-out flex flex-col gap-2 w-[100%]">
            <div className="flex w-full items-center justify-between">
                <span className="text-foreground-dark/80 text-md tracking-wide">Engine</span>
                {engineRunning ? (
                    <div className="inline-flex items-center gap-2 rounded-md bg-green-700/40 border border-green-500/60 px-3 py-1 cursor-pointer"
                        onClick={async (e) => {
                            e.preventDefault();
                            let stopped = await invoke<boolean>('stop_analyzer');
                            if (stopped) {
                                setEngineRunning(false);
                            }

                        }}>
                        <span className="relative flex h-2 w-2">
                            <span className="absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75 animate-ping"></span>
                            <span className="relative inline-flex rounded-full h-2 w-2 bg-green-400"></span>
                        </span>
                        <span className="text-green-300 text-xs font-semibold">Running</span>
                    </div>
                ) : (
                    <div className="inline-flex items-center gap-2 rounded-full bg-red-700/40 border border-red-500/60 px-3 py-1 cursor-pointer"
                        onClick={(e) => {
                            e.preventDefault();
                            startEngine();
                        }}>
                        <span className="inline-flex rounded-full h-2 w-2 bg-red-400"></span>
                        <span className="text-red-300 text-xs font-semibold">Stopped</span>
                    </div>
                )}
            </div>
            <div className="">
                {localPv && (
                    <span className="text-xs text-secondary">depth({localPv.depth})</span>
                )}
            </div>
            <div className="flex flex-col text-xs gap-3 w-[100%]">
                {

                    !localPv || localPv.depth === 0 || engineLoading ? (
                        <div className="flex items-center gap-2 text-secondary/80">
                            <LoaderCircle className="w-4 h-4 animate-spin" />
                            <span>loading engine lines</span>
                        </div>
                    ) : (
                        Object.entries(localPv.lines).map(([idx, line]) =>
                            line ? (
                                <div className="flex flex-row gap-2" key={idx}>
                                    <span
                                        className={`font-mono w-12 text-right px-1 rounded ${line.eval_kind === "Mate"
                                            ? line.eval_value > 0
                                                ? "bg-white text-black"
                                                : "bg-black text-white"
                                            : line.eval_value > 0
                                                ? "bg-white text-black"
                                                : "bg-black text-white"
                                            }`}
                                    >
                                        {line.eval_kind === "Mate"
                                            ? `${line.eval_value > 0 ? "+" : line.eval_value < 0 ? "-" : ""}M${Math.abs(line.eval_value)}`
                                            : (line.eval_value / 100).toFixed(2)}
                                    </span>
                                    <span
                                        className="inline-block max-w-full truncate text-foreground-dark/90"
                                        title={line.moves}
                                    >
                                        {line.moves}
                                    </span>
                                </div>
                            ) : null
                        )
                    )

                }

            </div>
        </div>
    )
}
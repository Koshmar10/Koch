import { Box, Divide, GitBranch, Layers, Settings } from "lucide-react";
import { Dispatch, SetStateAction, useEffect, useReducer, useRef, useState } from "react";
import { AnalyzerController } from "../../../src-tauri/bindings/AnalyzerController";
import { EngineOption } from "../../../src-tauri/bindings/EngineOption";
import { invoke } from "@tauri-apps/api/core";

interface Props {
    analyzer: AnalyzerController | null;
    setEngineLoading: Dispatch<SetStateAction<boolean>>
    openGameSelectPopup: () => void;
    startEngine: () => void;
}
type AnalyzerMode = "game" | "sandbox";
const sectionTitle = (Icon: React.ElementType, text: React.ReactNode) => {
    return (
        <div className="flex items-center gap-2 ">
            <Icon className="h-4 w-4 text-secondary/50" />
            <span className="text-xs uppercase tracking-wider text-secondary/60">{text}</span>
        </div>

    );
};

const loadAnalyzerModeContent = (mode: AnalyzerMode, analyzer: AnalyzerController | null, open?: () => void) => {
    if (mode == "game")
        if (analyzer?.game_id) {
            return (
                <div className="flex flex-col w-full gap-4">
                    <div className="game-card flex flex-col w-full text-sm bg-dark px-4 py-2 rounded-lg border-[1px] border-primary/60 hover:border-primary hover:cursor-pointer hover:text-primary transition-colors duration-100"
                        onClick={(e) => { e.preventDefault(); open && open(); }}>
                        <span className="mb-1 text-[0.92rem] hover:text-primary ">{analyzer.board.meta_data.white_player_name} Vs. {analyzer.board.meta_data.black_player_name}</span>
                        <span className="text-[0.8rem] text-secondary/80">{analyzer.board.meta_data.date} * {analyzer.board.meta_data.opening}</span>
                    </div>

                    <div className="flex flex-row gap-10 w-full">
                        <div className="flex flex-col text-xs gap-3">
                            <div className="flex flex-col">
                                <span className="uppercase tracking-wide text-secondary/60">White</span>
                                <span className="text-sm font-medium text-secondary">{analyzer.board.meta_data.white_player_name}</span>
                                <span className="text-[0.8rem] text-secondary/70">ELO: <span className="font-semibold text-secondary">{analyzer.board.meta_data.white_player_elo}</span></span>
                            </div>
                            <div className="flex flex-col">
                                <span className="uppercase tracking-wide text-secondary/60">Date</span>
                                <span className="text-sm font-medium text-secondary">{analyzer.board.meta_data.date}</span>
                            </div>
                        </div>
                        <div className="flex flex-col text-xs gap-3">
                            <div className="flex flex-col">
                                <span className="uppercase tracking-wide text-secondary/60">Black</span>
                                <span className="text-sm font-medium text-secondary">{analyzer.board.meta_data.black_player_name}</span>
                                <span className="text-[0.8rem] text-secondary/70">ELO: <span className="font-semibold text-secondary">{analyzer.board.meta_data.black_player_elo}</span></span>
                            </div>
                            <div className="flex flex-col">
                                <span className="uppercase tracking-wide text-secondary/60">Result</span>
                                <span className="text-sm font-medium text-secondary">{analyzer.board.meta_data.result}</span>
                            </div>
                        </div>
                    </div>

                </div>

            )
        }
        else {
            return (
                <div>
                    no game data
                </div>

            )
        }
    if (mode == "sandbox")
        return (
            <div>
                sand
            </div>
        )
}
export function AnalyzerSettings({ analyzer, setEngineLoading, startEngine, openGameSelectPopup }: Props) {
    const [selectedMode, setSelectedMode] = useState<"game" | "sandbox">("sandbox");
    const [sysMemory, setSysMemory] = useState<number>(0);
    const [sysCpu, setSysCpu] = useState<number>(1);
    const [changeingOption, setChangeingOption] = useState<boolean>(false);

    // State for current engine settings
    const [currentPv, setCurrentPv] = useState<number>(3);
    const [currentThreads, setCurrentThreads] = useState<number>(1);
    const [currentHash, setCurrentHash] = useState<number>(16);

    useEffect(() => {
        const getSysInfo = async () => {
            try {
                // Assuming returns [total_memory_bytes, cpu_cores]
                const [mem, cpu] = await invoke<[number, number]>('get_system_information');
                const [pv, th, hs] = await invoke<[number, number, number]>('get_analyzer_settings');

                setSysMemory(mem);
                setSysCpu(cpu);

                // Initialize settings from backend
                setCurrentPv(pv);
                setCurrentThreads(th);
                setCurrentHash(hs);
            } catch (e) {
                console.error("Failed to get system info", e);
            }
        }
        getSysInfo();
    }, [])

    useEffect(() => {
        if (analyzer && analyzer.game_id != -1) {
            setSelectedMode("game");
        }
        else {
            setSelectedMode("sandbox");
        }
    }, [analyzer])

    // Generate Thread Options (Powers of 2 up to sysCpu)
    const threadOptions = (() => {
        const options = [];
        let t = 1;
        while (t <= sysCpu) {
            options.push(t);
            t *= 2;
        }
        // Ensure the exact core count is available if not a power of 2
        if (options[options.length - 1] !== sysCpu) {
            options.push(sysCpu);
        }
        return options.sort((a, b) => a - b);
    })();

    // Generate Hash Options (Powers of 2, starting 16MB, up to ~70% of RAM)
    const hashOptions = (() => {
        const options = [];
        let h = 16; // Start at 16 MB

        // sysMemory is in GB. Convert to MB.
        // Safety cap at 70% of total RAM to avoid system freeze.
        const maxHash = (sysMemory * 1024) * 0.7;

        while (h <= maxHash) {
            options.push(h);
            h *= 2;
        }
        // If system has very low memory, ensure at least 16MB is there
        if (options.length === 0) return [16];
        return options;
    })();

    const changeOption = async (option: EngineOption, value: string) => {
        if (changeingOption) return; // Prevent double submission

        setEngineLoading(true)
        setChangeingOption(true);
        try {
            await invoke('set_engine_option', { option: option, value: value });
            // Give the backend a moment to restart the engine process if necessary
            await new Promise(r => setTimeout(r, 100));
            startEngine();
        } catch (e) {
            console.error(`Failed to set ${option}`, e);
        } finally {
            setChangeingOption(false);
            setEngineLoading(false);
        }
    }

    return (
        <div className="flex flex-col h-full w-[24%]">
            <div className="flex flex-col gap-2  bg-primary/20 border-b-2 border-r-2 border-accent/60 h-[30%] w-full p-4">
                {sectionTitle(Layers, "Analyzer Mode")}
                <div className="game-sandbox-buttons flex flex-row gap-4 w-full justify-center">
                    <button
                        className={`w-full py-1 rounded-md flex items-center justify-center gap-2 ${selectedMode == "sandbox" ? 'bg-accent' : 'bg-primary/30'}`}
                        onClick={() => setSelectedMode("sandbox")}
                    >
                        <Box className="h-4 w-4" />
                        Sandbox
                    </button>
                    <button
                        className={`w-full py-1 rounded-md flex items-center justify-center gap-2 ${selectedMode == "game" ? 'bg-accent' : 'bg-primary/30'}`}
                        onClick={() => setSelectedMode("game")}
                    >
                        <Layers className="h-4 w-4" />
                        Game
                    </button>
                </div>
                {loadAnalyzerModeContent(selectedMode, analyzer, openGameSelectPopup)}
            </div>
            <div className="bg-primary/20 border-b-2 border-r-2 border-accent/60 p-4 h-[28%]">
                {sectionTitle(Settings, "Engine Settings")}
                <div className="flex flex-col gap-3 w-full mt-4 justify-center items-center mb-4">
                    <div className="relative w-[95%]">
                        <label className="block text-xs text-secondary/70 mb-1">Multi-PV</label>
                        <select
                            className="w-full p-1 rounded bg-primary/80 text-secondary border border-accent/40 focus:outline-none focus:ring-2 focus:ring-accent transition text-sm appearance-none pr-8"
                            disabled={changeingOption}
                            value={currentPv}
                            onChange={(e) => {
                                const val = Number(e.target.value);
                                setCurrentPv(val);
                                changeOption("MultiPv" as EngineOption, e.target.value);
                            }}
                        >
                            <option value="1" className="bg-primary text-secondary">1 Variation</option>
                            <option value="2" className="bg-primary text-secondary">2 Variations</option>
                            <option value="3" className="bg-primary text-secondary">3 Variations</option>
                            <option value="4" className="bg-primary text-secondary">4 Variations</option>
                            <option value="5" className="bg-primary text-secondary">5 Variations</option>
                        </select>
                        <span className="pointer-events-none absolute right-2  text-secondary/60">
                            ▼
                        </span>
                    </div>
                    <div className="relative w-[95%]">
                        <label className="block text-xs text-secondary/70 mb-1">Threads</label>
                        <select
                            className="w-full p-1 rounded bg-primary/80 text-secondary border border-accent/40 focus:outline-none focus:ring-2 focus:ring-accent transition text-sm appearance-none pr-8"
                            disabled={changeingOption}
                            value={currentThreads}
                            onChange={(e) => {
                                const val = Number(e.target.value);
                                setCurrentThreads(val);
                                changeOption("Threads" as EngineOption, e.target.value);
                            }}
                        >
                            {threadOptions.map(t => (
                                <option key={t} value={t} className="bg-primary text-secondary">
                                    {t}
                                </option>
                            ))}
                        </select>
                        <span className="pointer-events-none absolute right-2  text-secondary/60">
                            ▼
                        </span>
                    </div>
                    <div className="relative w-[95%]">
                        <label className="block text-xs text-secondary/70 mb-1">Hash Size</label>
                        <select
                            className="w-full p-1 rounded bg-primary/80 text-secondary border border-accent/40 focus:outline-none focus:ring-2 focus:ring-accent transition text-sm appearance-none pr-8"
                            disabled={changeingOption}
                            value={currentHash}
                            onChange={(e) => {
                                const val = Number(e.target.value);
                                setCurrentHash(val);
                                changeOption("Hash" as EngineOption, e.target.value);
                            }}
                        >
                            {hashOptions.map(h => (
                                <option key={h} value={h} className="bg-primary text-secondary">
                                    {h} MB
                                </option>
                            ))}
                        </select>
                        <span className="pointer-events-none absolute right-2 text-secondary/60">
                            ▼
                        </span>
                    </div>
                </div>
            </div>
            <div className="bg-primary/20 border-b-2 border-r-2 border-accent/60 h-full p-4 ">
                {sectionTitle(GitBranch, "Move Timeline")}
                <div className="flex flex-row w-full mt-4">
                    <div className="flex flex-col items-end pr-2 text-xs text-secondary/60 min-w-[2rem]">
                        {
                            analyzer && analyzer.board.meta_data.move_list.length > 0 &&
                            Array.from({ length: Math.ceil(analyzer.board.meta_data.move_list.length / 2) }, (_, i) => (
                                <div key={i} className="h-6 flex items-center justify-end">{i + 1}.</div>
                            ))
                        }
                    </div>
                    <div className="flex flex-col w-full">
                        {
                            analyzer && analyzer.board.meta_data.move_list.map((mv, idx) =>
                                idx % 2 === 0 ? (
                                    <div
                                        key={idx}
                                        className={`h-6 flex items-center px-2 rounded text-white/80  text-sm font-medium 
                                            ${analyzer.current_ply == idx ? 'bg-accent/20' : 'hover:bg-accent/20'}`}
                                    >
                                        {mv.uci}
                                    </div>
                                ) : null
                            )
                        }
                    </div>
                    <div className="flex flex-col w-full">
                        {
                            analyzer && analyzer.board.meta_data.move_list.map((mv, idx) =>
                                idx % 2 === 1 ? (
                                    <div
                                        key={idx}
                                        className={`h-6 flex items-center px-2 rounded text-white/80  text-sm font-medium 
                                            ${analyzer.current_ply == idx ? 'bg-accent/20' : 'hover:bg-accent/20'}`}
                                    >
                                        {mv.uci}
                                    </div>
                                ) : null
                            )
                        }
                    </div>
                </div>
            </div>
        </div>
    )
}
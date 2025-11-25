import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react"
import { BoardMetaData } from "../../src-tauri/bindings/BoardMetaData";
import { SpaceIcon } from "lucide-react";


interface HistoryProps {
    onOpenGame?: (id: number) => void;
}

export function History({ onOpenGame }: HistoryProps) {
    const [pastGames, setPastGames] = useState<BoardMetaData[]>([])
    useEffect(() => {
        async function func() {

            const games = await invoke<BoardMetaData[]>("fetch_game_history");
            console.log(games);
            setPastGames(games);
        }
        func();
    }, [])
    return (
        <div className="flex flex-col w-[100%] justify-start gap-5">
            <div className="flex flex-col items-start justify-start gap-4 ">
                <div className="bg-primary/20 w-full flex flex-start">
                    <h1 className=" pt-4 pb-2 text-xl font-normal  ">History</h1>
                </div>
                <div>
                    <input type="text" name="" id="" />
                </div>
            </div>
            <div className="w-fill flex flex-row flex-wrap gap-4 p-4">
                {
                    pastGames.map((pg, idx) => (
                        <button
                            key={idx + 1}
                            onClick={() => onOpenGame && onOpenGame(idx + 1)} // replace idx with pg.id if you store real ids
                            className="text-left px-2 py-4 w-[300px] bg-primary/80 hover:bg-primary/60 rounded-md flex flex-col gap-2 transition"
                        >
                            <div className="w-[100%] flex felx-row justify-between items-center">
                                <span className="text-lg font-normal">{pg.white_player_name}({pg.white_player_elo})</span>
                                <span className="text-sm text-secondary/80 font-light">White</span>
                            </div>
                            <div className="w-[100%] flex felx-row justify-between items-center">
                                <span className="text-lg font-normal">{pg.black_player_name}({pg.black_player_elo})</span>
                                <span className="text-sm text-secondary/80 font-light">Black</span>
                            </div>
                            <div className="flex flex-start text-sm text-secondary/60 ">
                                <h1>Openingn name</h1>
                            </div>
                            <div className="w-100 h-[1px] bg-white/20" />
                            <div className="flex flex-col gap-1 text-sm">
                                <div className="flex items-center justify-between gap-2">
                                    <span className="px-2 py-0.5 rounded bg-secondary/20 text-secondary/90">
                                        {
                                            pg.result === "WhiteWin"
                                                ? "1-0"
                                                : pg.result === "BlackWin"
                                                    ? "0-1"
                                                    : pg.result === "Draw"
                                                        ? "1/2-1/2"
                                                        : "-"
                                        }
                                    </span>
                                    <span className="text-secondary/60">
                                        {
                                            (() => {
                                                const d = new Date(pg.date);
                                                return isNaN(d.getTime())
                                                    ? pg.date
                                                    : d.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
                                            })()
                                        }
                                    </span>
                                </div>
                            </div>
                            <div className="text-xs text-secondary/50">Open in Analyzer →</div>
                        </button>
                    ))
                }
            </div>
        </div >
    )

}
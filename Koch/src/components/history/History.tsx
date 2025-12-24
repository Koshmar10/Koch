import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react"
import { BoardMetaData } from "../../../src-tauri/bindings/BoardMetaData";
import { LoadPgnPopup } from "./LoadPgnPopup";
import { Settings } from "../../../src-tauri/bindings/Settings";



interface HistoryProps {
    onOpenGame?: (id: number) => void;
}

export function History({ onOpenGame }: HistoryProps) {
    const [pastGames, setPastGames] = useState<BoardMetaData[]>([])
    const [laodPgnPopupOpen, setLaodPgnPopupOpen] = useState<boolean>(false);
    const [reloadKey, setReloadKey] = useState<number>(0); // <- NEW: trigger for re-fetch
    const [chessdotcomSyncAllowed, setChessdotcomSyncAllowed] = useState<boolean>(false);

    useEffect(() => {
        async function func() {
            const games = await invoke<BoardMetaData[]>("fetch_game_history");
            setPastGames(games);
        }
        func();
    }, [reloadKey]) // <- RELOAD when reloadKey changes (e.g. after successful load)
    useEffect(() => {
        const get_user = async () => {
            try {
                // fix typo and safely convert to boolean
                const s = await invoke<Settings>('get_settings');
                const hasUser = Boolean(s.map && s.map["chessdotcom_user"]);
                setChessdotcomSyncAllowed(hasUser);
            } catch (err) {
                console.error("get_settings failed:", err);
                setChessdotcomSyncAllowed(false);
            }
        }
        get_user();
    }, [])

    const handleChessdotcomSync = async () => {
        if (!chessdotcomSyncAllowed) return;
        setChessdotcomSyncAllowed(false);
        try {
            await invoke<null>('sync_with_chessdotcom');
            // refresh list on success
            setReloadKey(k => k + 1);
        } catch (err) {
            console.error("sync_with_chessdotcom failed:", err);
        } finally {
            // re-enable the button regardless of success/failure
            setChessdotcomSyncAllowed(true);
        }
    }
    return (
        <div className="flex flex-col w-[100%] justify-start gap-5">
            <LoadPgnPopup
                popupOpen={laodPgnPopupOpen}
                setPopupOpen={setLaodPgnPopupOpen}
                onLoadSuccess={() => setReloadKey(k => k + 1)} // <- increment to refresh list
            />
            <div className="flex flex-col items-start justify-start gap-4 ">
                <div className="bg-primary/20 w-full flex flex-start">
                    <h1 className=" pt-4 pb-2 text-xl font-normal  ">History</h1>
                </div>
                <div>
                    <input type="text" name="" id="" />
                </div>
                <div className="bg-primary px-4 py-2 rounded-lg hover:cursor-pointer"
                    onClick={() => setLaodPgnPopupOpen(true)}>
                    Load PGN
                </div>
                <button className="bg-primary px-4 py-2 rounded-lg hover:cursor-pointer"
                    onClick={() => handleChessdotcomSync()}
                    disabled={!chessdotcomSyncAllowed}>
                    Sync with Chess.com
                </button>
            </div>
            <div className="w-fill flex flex-row flex-wrap gap-4 p-4 overflow-y-scroll">
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
                                <h1>{pg.opening}</h1>
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
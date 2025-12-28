import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { BoardMetaData } from "../../../src-tauri/bindings/BoardMetaData";
import { LoadPgnPopup } from "./LoadPgnPopup";
import { Settings } from "../../../src-tauri/bindings/Settings";
import { GameCard } from "./GameCard"; // Import the new GameCard component
import SearchInputs from "./SearchInputs";
import ActivityChart from "./ActivityChart";

interface HistoryProps {
    onOpenGame?: (id: number) => void;
}

export function History({ onOpenGame }: HistoryProps) {
    const [pastGames, setPastGames] = useState<BoardMetaData[]>([]);
    const [laodPgnPopupOpen, setLaodPgnPopupOpen] = useState<boolean>(false);
    const [reloadKey, setReloadKey] = useState<number>(0);
    const [chessdotcomSyncAllowed, setChessdotcomSyncAllowed] = useState<boolean>(false);

    useEffect(() => {
        async function func() {
            const games = await invoke<BoardMetaData[]>("fetch_game_history");
            setPastGames(games);
        }
        func();
    }, [reloadKey]);

    useEffect(() => {
        const get_user = async () => {
            try {
                const s = await invoke<Settings>("get_settings");
                const hasUser = Boolean(s.map && s.map["chessdotcom_user"]);
                setChessdotcomSyncAllowed(hasUser);
            } catch (err) {
                console.error("get_settings failed:", err);
                setChessdotcomSyncAllowed(false);
            }
        };
        get_user();
    }, []);

    const handleChessdotcomSync = async () => {
        if (!chessdotcomSyncAllowed) return;
        setChessdotcomSyncAllowed(false);
        try {
            await invoke<null>("sync_with_chessdotcom");
            setReloadKey((k) => k + 1);
        } catch (err) {
            console.error("sync_with_chessdotcom failed:", err);
        } finally {
            setChessdotcomSyncAllowed(true);
        }
    };

    return (
        <div className="flex flex-col w-[100%] justify-start">
            <LoadPgnPopup
                popupOpen={laodPgnPopupOpen}
                setPopupOpen={setLaodPgnPopupOpen}
                onLoadSuccess={() => setReloadKey((k) => k + 1)}
            />
            <div className="bg-card-dark/20 border-b-2 border-border-dark w-full flex flex-start">
                <h1 className="pl-2 pt-4 pb-4 text-xl font-normal text-foreground-dark">Game History</h1>
            </div>
            <ActivityChart />
            <SearchInputs onSyncClick={handleChessdotcomSync} onLoadClick={() => { setLaodPgnPopupOpen(true) }} />
            <div className="w-fill flex flex-row flex-wrap gap-4 p-4 overflow-y-scroll">
                {pastGames.map((pg, idx) => (
                    <GameCard
                        key={idx + 1}
                        game={pg}
                        onClick={() => onOpenGame && onOpenGame(idx + 1)} // Replace idx with pg.id if you store real IDs
                    />
                ))}
            </div>
        </div>
    );
}
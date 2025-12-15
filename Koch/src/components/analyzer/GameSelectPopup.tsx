import React, { Dispatch, SetStateAction, useEffect, useState } from "react";
import { GameResult } from "../../../src-tauri/bindings/GameResult";
import { BoardMetaData } from "../../../src-tauri/bindings/BoardMetaData";
import { invoke } from "@tauri-apps/api/core";
import { GameCard } from "./GameCard";

interface Props {
    isRendered: boolean;
    closePopup: Dispatch<SetStateAction<boolean>>;
    setSelectedGameId?: Dispatch<SetStateAction<number | null>>;
}



export function GameSelectPopup({ isRendered, closePopup, setSelectedGameId }: Props) {
    if (!isRendered) return null;

    const [pastGames, setPastGames] = useState<BoardMetaData[]>([]);

    useEffect(() => {
        async function func() {
            const games = await invoke<BoardMetaData[]>("fetch_game_history");
            setPastGames(games);
        }
        func();
    }, []);

    return (
        // Important: parent analyzer section must be `relative` so this absolute overlay only covers that area
        <div className="absolute inset-0 z-40 flex items-center justify-center bg-black/40 backdrop-blur-sm">
            <div className="relative bg-dark/95 border border-primary/40 rounded-lg shadow-xl
                            w-[min(720px,94%)] max-h-[80%] p-6 flex flex-col">
                {/* Header */}
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-lg font-bold text-secondary">Select a Game</h2>
                    <button
                        className="px-2.5 py-1.5 text-xs rounded bg-white/5 border border-white/10 text-secondary hover:bg-white/10 hover:border-white/20 transition"
                        onClick={() => closePopup(false)}
                        aria-label="Close"
                    >
                        Close
                    </button>
                </div>

                {/* Filters or helper text (optional) */}
                <div className="mb-3 text-[12px] text-secondary/60">
                    Choose a past game to load into the analyzer.
                </div>

                {/* List */}
                <div
                    className="flex-1 overflow-y-auto rounded-md border border-primary/30  p-3"
                    style={{
                        scrollbarWidth: "thin",
                    }}
                >
                    <div className="flex flex-col gap-2">
                        {pastGames.length === 0 ? (
                            <div className="text-sm text-secondary/60 py-8 text-center">
                                No past games found.
                            </div>
                        ) : (
                            pastGames.map((game, idx) =>
                                <GameCard
                                    key={`${idx + 1}`}
                                    onClick={() => {
                                        if (setSelectedGameId) setSelectedGameId(idx + 1);
                                        closePopup(false);
                                    }}
                                    title={`${game.white_player_name} (${game.white_player_elo}) vs ${game.black_player_name} (${game.black_player_elo})`}
                                    date={game.date}
                                    result={game.result}
                                />
                            )
                        )}
                    </div>
                </div>

                {/* Footer actions (optional) */}
                <div className="mt-4 flex items-center justify-end gap-2">
                    <button
                        className="px-3 py-1.5 text-xs rounded bg-white/5 border border-white/10 text-secondary hover:bg-white/10 hover:border-white/20 transition"
                        onClick={() => closePopup(false)}
                    >
                        Cancel
                    </button>
                    <button
                        className="px-3 py-1.5 text-xs rounded bg-accent text-white hover:bg-accent/80 transition"
                        onClick={() => closePopup(false)}
                    >
                        Load Selected
                    </button>
                </div>
            </div>
        </div>
    );
}
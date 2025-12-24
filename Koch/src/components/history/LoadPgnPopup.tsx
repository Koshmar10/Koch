import { invoke } from "@tauri-apps/api/core";
import { Dispatch, SetStateAction, useState } from "react";


interface Props {
    popupOpen: boolean;
    setPopupOpen: Dispatch<SetStateAction<boolean>>;
    onLoadSuccess?: () => void; // <- NEW: callback when load succeeds
}

export function LoadPgnPopup({ popupOpen, setPopupOpen, onLoadSuccess }: Props) {
    if (!popupOpen) return null;
    const [error, setError] = useState<string | null>(null);
    const [input, setInput] = useState<string>(""); // <- NEW: textarea state
    const handle_load = async (input_string: string) => {
        try {
            await invoke('load_pgn_game', { inputString: input_string }); // fixed param name
            onLoadSuccess && onLoadSuccess(); // notify parent to refresh
            setError(null);
            setPopupOpen(false);
        } catch (e) {
            setError(String(e));
        }
    }
    return (
        <div className="absolute inset-0 z-40 flex items-center justify-center bg-black/40 backdrop-blur-sm">
            <div className="bg-dark/95 border border-primary/40 rounded-lg shadow-xl w-[min(420px,94%)] max-h-[80%] p-6 flex flex-col items-center">
                <h2 className="text-lg font-bold text-secondary mb-4">Load PGN</h2>
                {error && <p className="text-sm text-red-500 mb-2">Error: {error}</p>}
                <textarea
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    className="w-full min-h-[120px] max-h-[240px] rounded-md bg-primary/80 text-secondary border border-primary/30 p-2 mb-4 resize-y focus:outline-none focus:ring-2 focus:ring-accent"
                    placeholder="Paste your PGN here..."
                />
                <div className="flex gap-3 w-full justify-end">
                    <button
                        className="px-4 py-2 rounded bg-white/10 border border-white/20 text-secondary hover:bg-white/20 transition"
                        onClick={(e) => { e.preventDefault(); setPopupOpen(false) }}>
                        Cancel
                    </button>
                    <button
                        disabled={!input.trim()}
                        className="px-4 py-2 rounded bg-accent text-white font-semibold hover:bg-accent/80 transition disabled:opacity-50 disabled:cursor-not-allowed"
                        onClick={async (e) => { e.preventDefault(); await handle_load(input.trim()); }}
                    >
                        Load
                    </button>
                </div>
            </div>
        </div>
    );
}
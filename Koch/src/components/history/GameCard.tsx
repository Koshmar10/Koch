import { BoardMetaData } from "../../../src-tauri/bindings/BoardMetaData";

interface GameCardProps {
    game: BoardMetaData;
    onClick: () => void;
}

export function GameCard({ game, onClick }: GameCardProps) {
    // Determine the tag color based on the game type
    // Map time control (in seconds) to game mode
    function mapTimeControlToMode(seconds: number): "Bullet" | "Blitz" | "Rapid" | "Classical" {
        if (seconds <= 60) return "Bullet";
        if (seconds <= 180) return "Blitz";
        if (seconds <= 600) return "Rapid";
        return "Classical";
    }

    const gameMode = game.time_control !== null ? mapTimeControlToMode(parseInt(game.time_control)) : null;
    const tagColors: Record<string, string> = {
        Bullet: "bg-red-700/30 text-red-400 border-[1px] border-red-500/40",
        Blitz: "bg-orange-700/30 text-orange-400 border-[1px] border-orange-500/40",
        Rapid: "bg-blue-700/30 text-blue-400 border-[1px] border-blue-500/40",
        Classical: "bg-purple-700/30 text-purple-400 border-[1px] border-purple-500/40",
    };

    return (
        <button
            onClick={onClick}
            className="text-left px-4 py-8 w-[360px] bg-card-dark/60 hover:bg-primary/40 rounded-lg flex flex-col gap-2 transition-colors duration-200 text-foreground-dark"
        >
            {/* Tag */}
            {gameMode &&
                <div className={`text-[10px] tracking-widest font-medium px-2 py-0.5 rounded border w-fit ${tagColors[gameMode]}`}>
                    {gameMode}
                </div>
            }

            {/* Player Info */}
            <div className="w-[100%] flex flex-row justify-between items-center">
                <span className="text-lg font-normal">{game.white_player_name} ({game.white_player_elo})</span>
                <span className="text-sm text-foreground-dark/60 font-light">White</span>
            </div>
            <div className="w-[100%] flex flex-row justify-between items-center">
                <span className="text-lg font-normal">{game.black_player_name} ({game.black_player_elo})</span>
                <span className="text-sm text-foreground-dark/60 font-light">Black</span>
            </div>

            {/* Opening */}
            <div className="flex flex-start text-sm text-secondary-dark/60">
                <h1>{game.opening}</h1>
            </div>

            {/* Divider */}
            <div className="w-100 h-[1px] bg-border-dark" />

            {/* Result and Date */}
            <div className="flex flex-col gap-1 text-sm">
                <div className="flex items-center justify-between gap-2">
                    <span className="px-2 py-0.5 rounded text-foreground-dark/90">
                        {
                            game.result === "WhiteWin"
                                ? "1-0"
                                : game.result === "BlackWin"
                                    ? "0-1"
                                    : game.result === "Draw"
                                        ? "1/2-1/2"
                                        : "-"
                        }
                    </span>
                    <span className="text-foreground-dark/60">
                        {
                            (() => {
                                const d = new Date(game.date);
                                return isNaN(d.getTime())
                                    ? game.date
                                    : d.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
                            })()
                        }
                    </span>
                </div>
            </div>
        </button>
    );
}
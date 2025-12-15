import { GameResult } from "../../../src-tauri/bindings/GameResult";
interface Props {
    title: string, date: string, result: GameResult, onClick?: () => void
}
export function GameCard({ title, date, result, onClick }: Props) {
    const getResultStyles = (r: GameResult) => {
        const v = String(r).toLowerCase();
        if (v.includes("white")) return "bg-green-500/15 text-green-300 border border-green-500/30";
        if (v.includes("black")) return "bg-red-500/15 text-red-300 border border-red-500/30";
        if (v.includes("draw")) return "bg-yellow-500/15 text-yellow-300 border border-yellow-500/30";
        return "bg-secondary/15 text-secondary border border-secondary/30";
    };

    const mapResult = (r: GameResult) => {
        const v = String(r).toLowerCase();
        if (v.includes("white")) return "1-0";
        if (v.includes("black")) return "0-1";
        if (v.includes("draw")) return "½-½";
        return String(r);
    };

    return (
        <button
            className="group w-full text-left rounded-md border border-primary/30 bg-primary/60 px-4 py-3 shadow-sm transition
                       hover:bg-primary/70 hover:border-primary/50 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-accent/40"
            onClick={onClick}>
            <div className="flex items-center justify-between gap-4">
                <div className="flex flex-col">
                    <span className="text-sm font-semibold text-secondary group-hover:text-white">
                        {title}
                    </span>
                    <span className="text-[11px] text-secondary/60">{date}</span>
                </div>
                <span className={`text-[11px] px-2 py-1 rounded ${getResultStyles(result)}`}>
                    {mapResult(result)}
                </span>
            </div>
        </button>
    );
}
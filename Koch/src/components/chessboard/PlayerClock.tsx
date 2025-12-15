import { Clock } from "lucide-react";

interface Props {
    isActive: boolean;
    timeMs: number; // Now controlled by parent
}

export function PlayerClock({ isActive, timeMs }: Props) {
    // Format time as MM:SS.d
    const formatTime = (ms: number) => {
        const totalSeconds = Math.floor(ms / 1000);
        const minutes = Math.floor(totalSeconds / 60);
        const seconds = totalSeconds % 60;
        const deciseconds = Math.floor((ms % 1000) / 100);

        const minStr = minutes.toString().padStart(2, '0');
        const secStr = seconds.toString().padStart(2, '0');

        // Only show deciseconds when time is low (e.g., under 20s)
        if (minutes === 0 && totalSeconds < 20) {
            return `${minStr}:${secStr}.${deciseconds}`;
        }
        return `${minStr}:${secStr}`;
    };

    const isLowTime = timeMs < 1000 * 30; // Red alert under 30s

    return (
        <div className={`
            relative overflow-hidden
            flex items-center gap-3 px-4 py-2 rounded-lg shadow-md border
            transition-all duration-300 ease-in-out
            ${isActive
                ? "bg-accent/10 border-accent shadow-accent/20 scale-105"
                : "bg-primary/40 border-white/5 opacity-80 grayscale-[0.5]"}
        `}>
            {/* Active Indicator Pulse */}
            {isActive && (
                <span className="absolute top-2 right-2 flex h-2 w-2">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-accent opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-accent"></span>
                </span>
            )}

            <div className={`p-2 rounded-md ${isActive ? "bg-accent/20 text-accent" : "bg-white/5 text-secondary/50"}`}>
                <Clock className="w-5 h-5" />
            </div>

            <div className="flex flex-col">
                <span className="text-[10px] uppercase tracking-wider font-semibold text-secondary/40">
                    Time Left
                </span>
                <span className={`text-2xl font-mono font-bold tabular-nums leading-none tracking-tight
                    ${isLowTime && isActive ? "text-red-400 animate-pulse" : "text-secondary"}
                `}>
                    {formatTime(timeMs)}
                </span>
            </div>
        </div>
    );
}
import { ChessArrow } from "./ChessArrow";

export type ArrowData = {
    from: string;
    to: string;
    color?: string;
    type: "engine" | "user" | "ghost";
}

interface ArrowLayerProps {
    arrows: ArrowData[];
    suggestion: ArrowData | null
    threat: ArrowData | null
    isFlipped?: boolean;
    squareSize: number;
    ghostArrows?: ArrowData[]
}
export type SquarePosition = {
    row: number;
    col: number;
}
export function ArrowLayer({ arrows, suggestion, threat, ghostArrows, isFlipped, squareSize }: ArrowLayerProps) {
    return (
        <svg className="absolute top-0 left-0 w-full h-full z-40 pointer-events-none">
            {arrows.map((arrow, index) => (
                <ChessArrow
                    key={`${arrow.from}-${arrow.to}-${index}`}
                    from={arrow.from}
                    to={arrow.to}
                    color={arrow.color}
                    isFlipped={isFlipped}
                    squareSize={squareSize}
                />
            ))}

            {/* draw single suggestion (if present) after user/engine arrows */}
            {suggestion && (
                <ChessArrow
                    key={`suggestion-${suggestion.from}-${suggestion.to}`}
                    from={suggestion.from}
                    to={suggestion.to}
                    // default highlight color if none provided
                    color={suggestion.color ?? "cyan"}
                    isFlipped={isFlipped}
                    squareSize={squareSize}
                />
            )}
            {
                threat && (
                    <ChessArrow
                        key={`threat-${threat.from}-${threat.to}`}
                        from={threat.from}
                        to={threat.to}
                        // default highlight color if none provided
                        color={threat.color ?? "cyan"}
                        isFlipped={isFlipped}
                        squareSize={squareSize}
                    />
                )
            }
            {
                ghostArrows &&
                ghostArrows.map((arrow, index) => (
                    <ChessArrow
                        key={`${arrow.from}-${arrow.to}-${index}`}
                        isGhost={true}
                        from={arrow.from}
                        to={arrow.to}
                        color={arrow.color}
                        isFlipped={isFlipped}
                        squareSize={squareSize}
                    />
                ))

            }

        </svg>
    );
}

export function squareFromCoords(absX: number, absY: number, squareSize: number): number[] {
    const layer = document.getElementById("arrow-layer") || document.querySelector(".relative.inline-block.select-none") as HTMLElement;
    const bounds = layer?.getBoundingClientRect();
    if (!bounds) return [0, 0];
    const relativeX = Math.abs(absX - bounds.x);
    const relativeY = Math.abs(absY - bounds.y);
    const col = Math.floor(relativeX / squareSize);
    const row = Math.floor(relativeY / squareSize);
    return [row, col];
}
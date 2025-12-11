
import { ChessArrow } from "./ChessArrow";

export type ArrowData = {
    from: string;
    to: string;
    color?: string;
    type: "engine" | "user";
}

interface ArrowLayerProps {
    arrows: ArrowData[];
    isFlipped?: boolean;
    squareSize: number;
}
export type SquarePosition = {
    row: number;
    col: number;
}
export function ArrowLayer({ arrows, isFlipped, squareSize }: ArrowLayerProps) {
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
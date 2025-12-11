
import type React from 'react';

interface BoardSquareProps {
    key: string
    size: number;
    dark: boolean;
    selected: boolean;
    quietMove: boolean;
    attackMove: boolean;
    captureMove: boolean;
    inCheck: boolean;
    handleMouseDown: (e: React.MouseEvent<HTMLDivElement>) => void
    handleMouseUp: (e: React.MouseEvent<HTMLDivElement>) => void
}
export function BoardSquare({ size, key, dark, inCheck, selected, quietMove, captureMove, attackMove, handleMouseDown, handleMouseUp }: BoardSquareProps) {
    const lightColor = "#f0d9b5";
    const darkColor = "#a37a58ff";
    const selectedColor = "#f6f669"; // selection highlight

    return (
        <div
            className={[
                "relative",
                attackMove
                    ? "bg-red-500"
                    : inCheck
                        ? "bg-red-200"
                        : selected
                            ? "bg-yellow-200"
                            : dark
                                ? "bg-[#a37a58]"
                                : "bg-[#f0d9b5]",
                selected ? "" : "",
            ].join(" ")}
            style={{ width: size, height: size }}
            key={key}
            onMouseDown={(e) => handleMouseDown(e)}
            onMouseUp={(e) => handleMouseUp(e)}
            onContextMenu={(e) => e.preventDefault()}
        >
            {quietMove && (
                <div
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-gray-500 opacity-85 z-30"
                    style={{
                        width: Math.max(8, Math.floor(size * 0.3)),
                        height: Math.max(8, Math.floor(size * 0.3)),
                    }}
                />
            )}
            {captureMove && (
                <div
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full border-[3px] border-gray-500/90 box-border z-30"
                    style={{
                        width: Math.max(12, Math.floor(size * 0.6)),
                        height: Math.max(12, Math.floor(size * 0.6)),
                    }}
                />
            )}
        </div>
    )
}
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
    tint: string | undefined;
    hover?: boolean;
    handleMouseDown: (e: React.MouseEvent<HTMLDivElement>) => void
    handleMouseUp: (e: React.MouseEvent<HTMLDivElement>) => void
}
export function BoardSquare({ size, key, dark, inCheck, selected, quietMove, captureMove, attackMove, tint, handleMouseDown, handleMouseUp, hover = false }: BoardSquareProps) {
    const lightColor = "#f0d9b5";
    const darkColor = "#a37a58ff";
    const selectedColor = "#f6f669"; // selection highlight

    // base size style (no background color here — keep base square classes)
    const baseStyle: React.CSSProperties = {
        width: size,
        height: size,
    };

    // Always render the base square color classes — overlay will sit on top.
    const baseBgClass = attackMove
        ? "bg-red-500"
        : inCheck
            ? "bg-red-200"
            : selected
                ? "bg-yellow-200"
                : dark
                    ? "bg-[#a37a58]"
                    : "bg-[#f0d9b5]";

    // Add white highlight if hover is true
    const highlightClass = hover ? "bg-lime-600/90" : "";

    // Combine base and highlight classes
    const combinedBgClass = [baseBgClass, highlightClass].filter(Boolean).join(" ");

    return (
        <div
            className={[
                "relative",
                combinedBgClass,
            ].join(" ")}
            style={baseStyle}
            key={key}
            onMouseDown={(e) => handleMouseDown(e)}
            onMouseUp={(e) => handleMouseUp(e)}
            onContextMenu={(e) => e.preventDefault()}
        >
            {/* Tint overlay sits ABOVE the base square color and below markers/pieces.
                Use backgroundColor RGBA (provided by caller) so it appears as a colored overlay. */}
            {tint && (
                <div
                    className="absolute inset-0 pointer-events-none rounded-sm"
                    style={{
                        backgroundColor: tint,
                        zIndex: 10,
                    }}
                />
            )}

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
import React from 'react';

interface ArrowProps {
    from: string; // "row-col", e.g., "6-4"
    to: string;   // "row-col", e.g., "4-4"
    color?: string;
    opacity?: number;
    isFlipped?: boolean;
    squareSize: number; // Added prop
}

function getSquareCenter(square: string, isFlipped: boolean, squareSize: number) {
    const [rowStr, colStr] = square.split('-');
    const row = parseInt(rowStr, 10);
    const col = parseInt(colStr, 10);

    // Array indices: 0,0 is Top Left.
    // White POV: x=col, y=row
    // Black POV: x=7-col, y=7-row
    const xIndex = isFlipped ? 7 - col : col;
    const yIndex = isFlipped ? 7 - row : row;

    return {
        x: xIndex * squareSize + squareSize / 2,
        y: yIndex * squareSize + squareSize / 2,
    };
}

export const ChessArrow: React.FC<ArrowProps> = ({
    from,
    to,
    color = "#ff9900",
    opacity = 1.0,
    isFlipped = false,
    squareSize,
}) => {
    const start = getSquareCenter(from, isFlipped, squareSize);
    const end = getSquareCenter(to, isFlipped, squareSize);

    const markerId = `arrowhead-${color.replace('#', '')}`;
    const strokeWidth = squareSize * 0.2; // Proportional width

    return (
        <>
            <defs>
                <marker
                    id={markerId}
                    markerWidth={2.5}
                    markerHeight={2.5}
                    refX={1.25}
                    refY={1.25}
                    orient="auto"
                >
                    <path d="M0,0 L2.5,1.25 L0,2.5" fill={color} />
                </marker>
            </defs>

            <line
                x1={start.x}
                y1={start.y}
                x2={end.x}
                y2={end.y}
                stroke={color}
                strokeWidth={strokeWidth}
                markerEnd={`url(#${markerId})`}
                opacity={opacity}
                strokeLinecap="round"
            />
        </>
    );
};

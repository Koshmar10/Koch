import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";
import { getImage } from "./utils";


interface PieceProps {
    size: number;
    row: number;
    col: number;
    color: PieceColor
    kind: PieceType
    is_ghost?: boolean
}

export function Piece({ size, row, col, color, kind, is_ghost = false }: PieceProps) {
    return (
        <img
            src={getImage(color, kind)}
            alt=""
            className={`absolute ${is_ghost ? 'opacity-60' : 'opacity-100'}`} // Removed Tailwind transition classes to use inline style for precision
            style={{
                zIndex: 10,
                width: size * 0.8,
                height: size * 0.8,
                top: 0,
                left: 0,
                // Use translate3d for GPU acceleration
                transform: `translate3d(${col * size + size * 0.1}px, ${row * size + size * 0.1}px, 0)`,
                // Increased duration to 300ms for smoother movement
                transition: "transform 250ms ease-in-out",
                willChange: 'transform',
            }}
            draggable={false}
        />
    )
}
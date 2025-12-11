import white_pawn from "../../assets/pieces/white_pawn.png";
import white_knight from "../../assets/pieces/white_knight.png";
import white_bishop from "../../assets/pieces/white_bishop.png";
import white_rook from "../../assets/pieces/white_rook.png";
import white_queen from "../../assets/pieces/white_queen.png";
import white_king from "../../assets/pieces/white_king.png";
import black_pawn from "../../assets/pieces/black_pawn.png";
import black_knight from "../../assets/pieces/black_knight.png";
import black_bishop from "../../assets/pieces/black_bishop.png";
import black_rook from "../../assets/pieces/black_rook.png";
import black_queen from "../../assets/pieces/black_queen.png";
import black_king from "../../assets/pieces/black_king.png";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";

export const PIECE_IMAGES: Record<PieceColor, Record<PieceType, string>> = {
    White: {
        Pawn: white_pawn,
        Knight: white_knight,
        Bishop: white_bishop,
        Rook: white_rook,
        Queen: white_queen,
        King: white_king,
    },
    Black: {
        Pawn: black_pawn,
        Knight: black_knight,
        Bishop: black_bishop,
        Rook: black_rook,
        Queen: black_queen,
        King: black_king,
    },
};
export function getImage(color: PieceColor, kind: PieceType): string {
    return PIECE_IMAGES[color][kind];
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
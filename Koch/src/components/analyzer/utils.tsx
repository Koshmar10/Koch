import { PieceMoves } from "../../../src-tauri/bindings/PieceMoves"
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
export type MoveChache = {
    [x: number]: PieceMoves | undefined;
}


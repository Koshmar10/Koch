import { ChessPiece } from "../../../src-tauri/bindings/ChessPiece";
import { GhostPiece, RenderedPiece } from "./Chessboard";
import { Piece } from "./Piece";

interface Props {
    optimisticPieces: RenderedPiece[];
    flipped: boolean;
    squareSize: number;
    ghostPieces?: GhostPiece[];
}

export function PieceLayer({ optimisticPieces, flipped, squareSize, ghostPieces = [] }: Props) {
    const getVisualCoords = (r: number, c: number) => {
        return flipped ? [7 - r, 7 - c] : [r, c];
    };
    return (<>
        {optimisticPieces.map(({ piece, r, c, to_render }) => {
            if (!to_render) return null;

            const [visualR, visualC] = getVisualCoords(r, c);
            return (
                <Piece
                    key={piece.id}
                    size={squareSize}
                    row={visualR}
                    col={visualC}
                    kind={piece.kind}
                    color={piece.color}
                />
            );
        })}
        {ghostPieces.map(({ kind, color, r, c }, idx) => {

            const [visualR, visualC] = getVisualCoords(r, c);
            return (
                <Piece
                    key={idx}
                    size={squareSize}
                    row={visualR}
                    col={visualC}
                    kind={kind}
                    color={color}
                    is_ghost={true}
                />
            );
        })}

    </>)
}

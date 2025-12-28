import { Board } from "../src-tauri/bindings/Board";
import { SerializedBoard } from "../src-tauri/bindings/SerializedBoard";
import { ChessPiece } from "../src-tauri/bindings/ChessPiece";
import { PieceColor } from "../src-tauri/bindings/PieceColor";
import { PieceType } from "../src-tauri/bindings/PieceType";
import { PieceMoves } from "../src-tauri/bindings/PieceMoves";
import { MoveStruct } from "../src-tauri/bindings/MoveStruct";
import { SerializedAnalyzerController } from "../src-tauri/bindings/SerializedAnalyzerController";
import { AnalyzerController } from "../src-tauri/bindings/AnalyzerController";

export function deserialzer(board_data: SerializedBoard): Board {
    // 1. Parse the Piece Index Mapper
    // Format: "rowcol:id|rowcol:id" -> Map { "00" => 12, "01" => 15 }
    const idMap = new Map<string, number>();
    if (board_data.piece_index_mapper && board_data.piece_index_mapper.length > 0) {
        const entries = board_data.piece_index_mapper.split('|');
        for (const entry of entries) {
            const [coords, idStr] = entry.split(':');
            idMap.set(coords, parseInt(idStr, 10));
        }
    }

    // 2. Parse FEN to reconstruct squares and game state
    const fenParts = board_data.fen.split(' ');
    const boardStr = fenParts[0];
    const turnStr = fenParts[1];
    const castlingStr = fenParts[2];
    const enPassantStr = fenParts[3];
    const halfMoveStr = fenParts[4];
    const fullMoveStr = fenParts[5];

    const squares: (ChessPiece | null)[][] = [];
    const rows = boardStr.split('/');

    // FEN ranks go from 8 (index 0) down to 1 (index 7)
    rows.forEach((rowStr, rIdx) => {
        const rowSquares: (ChessPiece | null)[] = [];
        let cIdx = 0;

        for (const char of rowStr) {
            if (/\d/.test(char)) {
                // Empty squares
                const emptyCount = parseInt(char, 10);
                for (let i = 0; i < emptyCount; i++) {
                    rowSquares.push(null);
                    cIdx++;
                }
            } else {
                // Piece
                const isWhite = char === char.toUpperCase();
                const color: PieceColor = isWhite ? "White" : "Black";
                const lowerChar = char.toLowerCase();
                let kind: PieceType;

                switch (lowerChar) {
                    case 'p': kind = "Pawn"; break;
                    case 'n': kind = "Knight"; break;
                    case 'b': kind = "Bishop"; break;
                    case 'r': kind = "Rook"; break;
                    case 'q': kind = "Queen"; break;
                    case 'k': kind = "King"; break;
                    default: kind = "Pawn";
                }

                // Retrieve the original ID from the mapper using coordinates
                const idKey = `${rIdx}${cIdx}`;
                const id = idMap.get(idKey) || 0;

                rowSquares.push({
                    id,
                    color,
                    kind,
                    position: [rIdx, cIdx]
                } as ChessPiece);
                cIdx++;
            }
        }
        squares.push(rowSquares);
    });

    // 3. Parse Move Cache
    // Format: "id:Q1234C56A78|..."
    const move_cache: Record<number, PieceMoves> = {};

    if (board_data.piece_moves && board_data.piece_moves.length > 0) {
        const moveEntries = board_data.piece_moves.split('|');

        for (const entry of moveEntries) {
            const [idStr, movesData] = entry.split(':');
            const id = parseInt(idStr, 10);

            // Find delimiters
            const qIdx = movesData.indexOf('Q');
            const cIdx = movesData.indexOf('C');
            const aIdx = movesData.indexOf('A');

            // Helper to parse coordinate strings "1234" -> [[1,2], [3,4]]
            const parseCoords = (str: string): [number, number][] => {
                const res: [number, number][] = [];
                for (let i = 0; i < str.length; i += 2) {
                    const r = parseInt(str[i], 10);
                    const c = parseInt(str[i + 1], 10);
                    res.push([r, c]);
                }
                return res;
            };

            const quietStr = movesData.substring(qIdx + 1, cIdx);
            const captureStr = movesData.substring(cIdx + 1, aIdx);
            const attackStr = movesData.substring(aIdx + 1);

            move_cache[id] = {
                quiet_moves: parseCoords(quietStr),
                capture_moves: parseCoords(captureStr),
                attacks: parseCoords(attackStr)
            };
        }
    }

    // 4. Parse En Passant Target
    let en_passant_target: [number, number] | null = null;
    if (enPassantStr !== '-') {
        const fileMap: Record<string, number> = { 'a': 0, 'b': 1, 'c': 2, 'd': 3, 'e': 4, 'f': 5, 'g': 6, 'h': 7 };
        const file = fileMap[enPassantStr[0]];
        // FEN rank is 1-based, board index is 0-based (reversed)
        const rank = 8 - parseInt(enPassantStr[1], 10);
        en_passant_target = [rank, file];
    }

    // 5. Parse Metadata (Move List)
    // The serializer sends "e2e4 e7e5" string. We need to convert back to MoveStruct[]
    // Note: We only have the UCI string here, so we create a partial MoveStruct.
    const moveList: MoveStruct[] = [];
    if (board_data.meta_data.move_list && board_data.meta_data.move_list.length > 0) {
        const entries = board_data.meta_data.move_list.split(' ');
        let moveNumber = 1;

        const promoMap: Record<string, PieceType> = {
            q: "Queen",
            r: "Rook",
            b: "Bishop",
            n: "Knight",
        };

        for (const entry of entries) {
            if (!entry) continue;
            const parts = entry.split('|');
            // Expect: uci|san|clock|time_stamp|nag|annotation
            const uci = parts[0] ?? '';
            const sanPart = parts[1] ?? '';
            const clockPart = parts[2] ?? null;
            const timeStampPart = parts[3] ?? '';
            const nagPart = parts[4] ?? null;
            const annotationPart = parts[5] ?? null;

            // determine promotion from UCI (e.g. e7e8q)
            let promotion: PieceType | null = null;
            if (uci && uci.length >= 5) {
                const possiblePromo = uci[uci.length - 1].toLowerCase();
                if (promoMap[possiblePromo]) promotion = promoMap[possiblePromo];
            }

            const time_stamp = Number(timeStampPart);
            const nag = nagPart && nagPart !== '-' ? (isNaN(Number(nagPart)) ? nagPart : Number(nagPart)) : null;
            const clock = clockPart && clockPart !== '-' ? clockPart : null;
            const san = sanPart && sanPart !== '-' ? sanPart : uci;

            moveList.push({
                move_number: moveNumber++,
                san,
                uci,
                promotion,
                is_capture: san.includes('x'),
                time_stamp: !isNaN(time_stamp) ? time_stamp : Date.now(),
                clock,
                annotation: annotationPart,
                nag,
            } as MoveStruct);
        }
    }

    // 6. Construct final Board object
    return {
        squares: squares as Board["squares"],
        move_cache,
        turn: turnStr === 'w' ? "White" : "Black",
        en_passant_target,
        halfmove_clock: parseInt(halfMoveStr, 10) || 0,
        fullmove_number: parseInt(fullMoveStr, 10) || 1,
        white_big_castle: castlingStr.includes('Q'),
        black_big_castle: castlingStr.includes('q'),
        white_small_castle: castlingStr.includes('K'),
        black_small_castle: castlingStr.includes('k'),
        game_phase: board_data.game_phase,
        meta_data: {
            move_list: moveList,
            starting_position: board_data.meta_data.starting_position,
            date: board_data.meta_data.date,
            termination: board_data.meta_data.termination,
            result: board_data.meta_data.result,
            white_player_elo: board_data.meta_data.white_player_elo,
            black_player_elo: board_data.meta_data.black_player_elo,
            white_player_name: board_data.meta_data.white_player_name,
            black_player_name: board_data.meta_data.black_player_name,
            opening: board_data.meta_data.opening,
            event: board_data.meta_data.event ?? null,
            site: board_data.meta_data.site ?? null,
            round: board_data.meta_data.round ?? null,
            time_control: board_data.meta_data.time_control ?? null,
            end_time: board_data.meta_data.end_time ?? null,
            link: board_data.meta_data.link ?? null,
            eco: board_data.meta_data.eco ?? null,
        },
        ui: {
            pov: "White",
            white_taken: [],
            black_taken: [],
        },
        been_modified: false,
        next_id: 0,
        ply_count: 0
    };
}

const files = "abcdefgh";

function squareFromCoords([row, col]: [number, number]): string {
    return `${files[col]}${8 - row}`;
}

export function toUciMove(
    from: [number, number],
    to: [number, number],
    promotion?: PieceType | string
): string {
    const base = `${squareFromCoords(from)}${squareFromCoords(to)}`;
    if (!promotion) return base;

    const promoMap: Record<string, string> = {
        Queen: "q",
        Rook: "r",
        Bishop: "b",
        Knight: "n",
    };

    const promo = promoMap[promotion as string] ?? "";

    return promo ? `${base}${promo}` : base;
}
export function fromUciMove(uci: string): [[number, number], [number, number]] {
    const files = "abcdefgh";
    const fromFile = files.indexOf(uci[0]);
    const fromRank = 8 - parseInt(uci[1], 10);
    const toFile = files.indexOf(uci[2]);
    const toRank = 8 - parseInt(uci[3], 10);
    return [
        [fromRank, fromFile],
        [toRank, toFile]
    ];
}
export function deserializeAnalyzerController(data: SerializedAnalyzerController): AnalyzerController {
    return {
        game_id: data.game_id,
        board: deserialzer(data.serialized_board),
        current_ply: data.current_ply,
        board_undo: data.board_undo,
        last_threat: null,
        last_pv: null,
        chat_history: null
    }
}
export function fileRankToRowCol(square: string): [number, number] {
    const files = "abcdefgh";
    const file = square[0];
    const rank = square[1];
    const col = files.indexOf(file);
    const row = 8 - parseInt(rank, 10);
    if (col < 0 || col > 7 || row < 0 || row > 7 || isNaN(row)) {
        throw new Error(`Invalid square notation: "${square}"`);
    }
    return [row, col];
}



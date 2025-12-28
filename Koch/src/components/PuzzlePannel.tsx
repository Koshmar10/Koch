import { RefreshCcw, RefreshCw, RefreshCwIcon } from "lucide-react";
import { useEffect, useState } from "react";



export function PuzzlePannel() {
    const [puzzleData, setPuzzleData] = useState<string | null>();
    useEffect(() => {

    }, [])
    return <div>

        <span>Puzzle Section</span>

        <button>
            <RefreshCwIcon />
        </button>
        <span>{puzzleData}</span>
    </div>
}
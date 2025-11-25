import { AppScreen } from "../App";
import { ChessBoard } from "./Chessboard";
import { Home } from "./Home";
import { Pve } from "./Pve";
import { Analyzer } from "./Analyzer";
import { History } from "./History";

interface CenterSectionProps {
    selectedScreen: AppScreen;
    selectedGameId?: number | null;
    openGameInAnalyzer?: (id: number) => void;
}

export function CenterSection({ selectedScreen, selectedGameId, openGameInAnalyzer }: CenterSectionProps) {
    switch (selectedScreen) {
        case "Home":
            return <Home />
        case "Pve":
            return <Pve />
        case "Analyzer":
            return <Analyzer gameId={selectedGameId} />
        case "History":
            return <History onOpenGame={openGameInAnalyzer} />
        default:
            return <Home />
    }

}
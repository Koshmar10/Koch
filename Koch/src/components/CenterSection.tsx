import { AppScreen } from "../App";
import { Home } from "./Home";
import { Pve } from "./Pve";
import { Analyzer } from "./analyzer/Analyzer";
import { History } from "./History";
import { Dispatch, SetStateAction } from "react";

interface CenterSectionProps {
    selectedScreen: AppScreen;
    selectedGameId?: number | null;
    openGameInAnalyzer?: (id: number) => void;
    setSelectedGameId?: Dispatch<SetStateAction<number | null>>
}

export function CenterSection({ selectedScreen, selectedGameId, openGameInAnalyzer, setSelectedGameId }: CenterSectionProps) {

    switch (selectedScreen) {
        case "Home":
            return <Home />
        case "Pve":
            return <Pve />
        case "Analyzer":
            return <Analyzer gameId={selectedGameId} setSelectedGameId={setSelectedGameId} />
        case "History":
            return <History onOpenGame={openGameInAnalyzer} />
        default:
            return <Home />
    }

}
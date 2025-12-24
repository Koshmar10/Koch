import { SetStateAction, useState } from "react";

import "./App.css";
import { Sidebar } from "./components/Sidebar";
import { CenterSection } from "./components/CenterSection";
import { NotificationManager } from "./components/notifications/NotificationManager";

export type AppScreen = "Home" | "Analyzer" | "History" | "Pve" | "Settings";
export const removeDefaultForButton = "border-0 p-0 m-0 appearance-none focus:outline-none"

function App() {

  const [centerSection, setCenterSection] = useState<AppScreen>("Home");
  const [selectedGameId, setSelectedGameId] = useState<number | null>(null);

  function changeSection(screen: AppScreen) {
    setCenterSection(screen);
  }

  function openGameInAnalyzer(id: number) {
    setSelectedGameId(id);
    setCenterSection("Analyzer");
  }

  return (
    <main className="bg-dark">
      <div className="flex flex-row w-[100vw] h-[100vh] ">

        <Sidebar handleClick={changeSection} selectedScreen={centerSection} />
        <CenterSection
          selectedScreen={centerSection}
          selectedGameId={selectedGameId}
          openGameInAnalyzer={openGameInAnalyzer} setSelectedGameId={setSelectedGameId} />
      </div>
    </main>
  );
}

export default App;

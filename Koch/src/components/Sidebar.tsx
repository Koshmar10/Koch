import { AppScreen } from "../App"
import { Home, Play, LineChart, History, Settings } from "lucide-react";

interface SideBarProps {
    handleClick: (screen: AppScreen) => void;
    selectedScreen: AppScreen;
}

export function Sidebar({ handleClick, selectedScreen }: SideBarProps) {
    const computeStyle = (screen: AppScreen) => {
        const base =
            "flex justify-center items-center w-fit h-fit text-secondary/90 py-3 px-3 rounded-lg transition-colors duration-300 ease-in-out";
        return screen === selectedScreen
            ? `${base} bg-accent/80 hover:bg-accent/90`
            : `${base} hover:bg-accent/50`;
    };

    return (
        <div className="flex flex-col w-[5%] border-r-[1px] border-white/20 gap-6">
            <div className="flex justify-center items-center pt-8 border-b-[1px] border-white/20 pb-6">
                <span className="w-fit h-fit px-5 py-3 rounded-lg text-secondary bg-accent text-xl">K</span>
            </div>

            <div className="h-full flex flex-col px-2 gap-4 items-center">

                <button
                    className={computeStyle("Home")}
                    aria-label="Home"
                    onClick={() => handleClick("Home")}
                >
                    <Home className="w-6 h-6" />
                </button>

                <button
                    className={computeStyle("Pve")}
                    aria-label="Play"
                    onClick={() => handleClick("Pve")}
                >
                    <Play className="w-6 h-6" />
                </button>

                <button
                    className={computeStyle("Analyzer")}
                    aria-label="Analysis"
                    onClick={() => handleClick("Analyzer")}
                >
                    <LineChart className="w-6 h-6" />
                </button>

                <button
                    className={computeStyle("History")}
                    aria-label="History"
                    onClick={() => handleClick("History")}
                >
                    <History className="w-6 h-6" />
                </button>


            </div>
            <div className="flex justify-center h-auto p-5 border-t-[1px] border-white/20 ">
                <button
                    className={computeStyle("Settings")}
                    aria-label="Settings"
                    onClick={() => handleClick("Settings")}
                >
                    <Settings />
                </button>
            </div>
        </div >
    );
}
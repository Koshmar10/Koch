import { AppScreen } from "../App"

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

            <div className="flex flex-col px-2 gap-4 items-center">
                <div
                    className={`${computeStyle("Home")}`}
                    aria-label="Home"
                    onClick={() => handleClick("Home")}
                >
                    <svg viewBox="0 0 24 24" className="w-6 h-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M3 11.5 12 4l9 7.5" />
                        <path d="M5 10v10h14V10" />
                        <path d="M9 14h6v6H9z" />
                    </svg>
                </div>

                <div
                    className={computeStyle("Pve")}
                    aria-label="Play"
                    onClick={() => handleClick("Pve")}
                >
                    <svg viewBox="0 0 24 24" className="w-6 h-6" fill="currentColor">
                        <path d="M6 4v16l14-8L6 4z" />
                    </svg>
                </div>

                <div
                    className={computeStyle("Analyzer")}
                    aria-label="Analysis"
                    onClick={() => handleClick("Analyzer")}
                >
                    <svg viewBox="0 0 24 24" className="w-6 h-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M4 19h16" />
                        <path d="M8 19V8" />
                        <path d="M12 19V4" />
                        <path d="M16 19v-6" />
                    </svg>
                </div>

                <div
                    className={computeStyle("History")}
                    aria-label="History"
                    onClick={() => handleClick("History")}
                >
                    <svg viewBox="0 0 24 24" className="w-6 h-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <circle cx="12" cy="12" r="9" />
                        <path d="M12 7v5l3 3" />
                    </svg>
                </div>
            </div>
        </div>
    );
}
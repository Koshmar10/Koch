import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react"
import { TrendingUp, Trophy, Target, Medal, Zap, Clock, Gamepad2, Bot, Puzzle, ChartColumn } from "lucide-react";
import { BottomCard } from "./BottomCard";

export function Home() {
    const [chessQuote, setChessQuote] = useState<string | null>(null)
    useEffect(() => {
        async function fetchQuote() {
            // Example API call, replace with your actual endpoint
            const quote = await invoke<string | null>('get_quote')
            setChessQuote(quote)
        }
        fetchQuote();
    }, []);

    const cardStyle = "bg-card-dark/40 hover:bg-card-dark/60 text-foreground-dark"
    const iconContainer = "mb-3 p-3 rounded-xl bg-primary-dark/15 w-fit"

    return (
        <div className="flex flex-col justify-start items-center h-[100vh] w-full bg-background-dark/100">
            <h1 className="text-3xl mt-4 mb-6 text-foreground-dark">Welcome back, player</h1>
            {chessQuote &&
                <div className="text-md text-center text-foreground-dark/60  italic mb-1 w-[90%] self-center">
                    {chessQuote}
                </div>
            }
            <div className="flex flex-col justify-center items-center w-full h-full">
                <div className="grid grid-cols-4 grid-rows-4 gap-4  w-[65%] h-[50rem] pt-20 px-4">

                    {/* 1. Current Rating */}
                    <div className={`rounded-lg shadow-sm row-span-2 col-span-1 flex flex-col justify-end items-center`}>
                        <div className={`h-[50%] w-full rounded-lg p-4 ${cardStyle} flex flex-col justify-between`}>
                            <div className={iconContainer}>
                                <TrendingUp size={20} className="h-5 w-5 text-primary-dark" strokeWidth={1.5} />
                            </div>
                            <div>
                                <div className="text-2xl font-bold text-foreground-dark">2145</div>
                                <div className="text-xs text-secondary-dark/70">Current Rating</div>
                                <div className="text-[10px] text-green-500 mt-1">+32 this month</div>
                            </div>
                        </div>
                    </div>

                    {/* 2. Avg Accuracy */}
                    <div className={`rounded-lg shadow-sm p-4 row-span-2 col-span-1 ${cardStyle} flex flex-col justify-start`}>
                        <div className={iconContainer}>
                            <Medal size={20} className="h-5 w-5 text-primary-dark" />
                        </div>
                        <div>
                            <div className="text-3xl font-bold text-foreground-dark">87.3%</div>
                            <div className="text-sm text-secondary-dark/70">Avg. Accuracy</div>
                        </div>
                    </div>

                    {/* 3. Win Rate */}
                    <div className={`rounded-lg shadow-sm p-4 row-span-1 col-span-2 ${cardStyle} flex flex-col justify-center`}>
                        <div className="flex flex-row items-center justify-between px-2">
                            <div>
                                <div className={iconContainer}>
                                    <Trophy size={20} className="h-5 w-5 text-primary-dark" />
                                </div>
                                <div className="text-2xl font-bold text-foreground-dark">64%</div>
                                <div className="text-xs text-secondary-dark/70">Win Rate</div>
                                <div className="text-[10px] text-green-500 mt-1">+5%</div>
                            </div>
                            {/* Decorative element or graph could go here */}
                        </div>
                    </div>

                    {/* 4. Total Time */}
                    <div className={` rounded-lg shadow-sm p-4 row-span-2 col-span-1 ${cardStyle} flex flex-col justify-start`}>
                        <div className={iconContainer}>
                            <Clock size={20} className="h-5 w-5 text-primary-dark" />
                        </div>
                        <div>
                            <div className="text-3xl font-bold text-foreground-dark">48h</div>
                            <div className="text-sm text-secondary-dark/70">Total Time</div>
                        </div>
                    </div>

                    {/* 5. Best Streak */}
                    <div className={`rounded-lg shadow-sm row-span-2 col-span-1 flex flex-col justify-start items-center `}>
                        <div className={`h-[50%] w-full rounded-lg p-4 ${cardStyle} flex flex-col justify-between`}>
                            <div className={iconContainer}>
                                <Zap size={20} className="h-5 w-5 text-primary-dark" />
                            </div>
                            <div>
                                <div className="text-2xl font-bold text-foreground-dark">12</div>
                                <div className="text-xs text-secondary-dark/70">Best Streak</div>
                                <div className="text-[10px] text-green-500 mt-1">wins</div>
                            </div>
                        </div>
                    </div>

                    {/* 6. Games Played */}
                    <div className={`rounded-lg shadow-sm p-4 row-span-1 col-span-2 ${cardStyle} flex flex-col justify-center`}>
                        <div className="flex flex-col px-2">
                            <div className={iconContainer}>
                                <Target size={20} className="h-5 w-5 text-primary-dark" />
                            </div>
                            <div className="text-2xl font-bold text-foreground-dark">127</div>
                            <div className="text-xs text-secondary-dark/70">Games Played</div>
                        </div>
                    </div>
                </div>
                <div className="flex flex-row items-center justify-between w-[65%] px-4">
                    <BottomCard icon={<Gamepad2 />} text={"New Game"} />
                    <BottomCard icon={<Bot />} text={"Vs Engine"} />
                    <BottomCard icon={<Puzzle />} text={"Puzzles"} />
                    <BottomCard icon={<TrendingUp />} text={"Progress"} />
                    <BottomCard icon={<ChartColumn />} text={"Analyzer"} />
                    <BottomCard icon={<Clock />} text={"History"} />
                </div>
            </div>
        </div>
    )

}
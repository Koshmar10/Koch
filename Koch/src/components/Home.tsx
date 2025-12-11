import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react"
import { TrendingUp, Trophy, Target, Medal, Zap, Clock } from "lucide-react";

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

    const cardColor = "bg-primary/20 border-2 border-accent/10"
    const iconContainer = "bg-accent/10 p-2 rounded-lg w-fit mb-2 text-accent"

    return (
        <div className="flex flex-col justify-start items-center h-[100vh] w-full ">
            <h1 className="text-3xl mt-4 mb-6">Welcome back, player</h1>
            {chessQuote &&
                <div className="text-md text-center text-secondary w-[90%] self-center">
                    {chessQuote}
                </div>
            }
            <div className="flex justify-center items-center w-full h-full text-secondary">
                <div className="grid grid-cols-4 grid-rows-4 gap-4  w-[50%] h-[50rem] pt-20 px-4">

                    {/* 1. Current Rating */}
                    <div className={`rounded-lg shadow-sm row-span-2 col-span-1 flex flex-col justify-end items-center `}>
                        <div className={`h-[50%] w-full rounded-lg p-4 ${cardColor} flex flex-col justify-between`}>
                            <div className={iconContainer}>
                                <TrendingUp size={20} />
                            </div>
                            <div>
                                <div className="text-2xl font-bold">2145</div>
                                <div className="text-xs text-secondary/70">Current Rating</div>
                                <div className="text-[10px] text-green-500 mt-1">+32 this month</div>
                            </div>
                        </div>
                    </div>

                    {/* 2. Avg Accuracy */}
                    <div className={`rounded-lg shadow-sm p-4 row-span-2 col-span-1 ${cardColor} flex flex-col justify-start`}>
                        <div className={iconContainer}>
                            <Medal size={20} />
                        </div>
                        <div>
                            <div className="text-3xl font-bold">87.3%</div>
                            <div className="text-sm text-secondary/70">Avg. Accuracy</div>
                        </div>
                    </div>

                    {/* 3. Win Rate */}
                    <div className={`rounded-lg shadow-sm p-4 row-span-1 col-span-2 ${cardColor} flex flex-col justify-center`}>
                        <div className="flex flex-row items-center justify-between px-2">
                            <div>
                                <div className={iconContainer}>
                                    <Trophy size={20} />
                                </div>
                                <div className="text-2xl font-bold">64%</div>
                                <div className="text-xs text-secondary/70">Win Rate</div>
                                <div className="text-[10px] text-green-500 mt-1">+5%</div>
                            </div>
                            {/* Decorative element or graph could go here */}
                        </div>
                    </div>

                    {/* 4. Total Time */}
                    <div className={` rounded-lg shadow-sm p-4 row-span-2 col-span-1 ${cardColor} flex flex-col justify-start`}>
                        <div className={iconContainer}>
                            <Clock size={20} />
                        </div>
                        <div>
                            <div className="text-3xl font-bold">48h</div>
                            <div className="text-sm text-secondary/70">Total Time</div>
                        </div>
                    </div>

                    {/* 5. Best Streak */}
                    <div className={`rounded-lg shadow-sm row-span-2 col-span-1 flex flex-col justify-start items-center `}>
                        <div className={`h-[50%] w-full rounded-lg p-4 ${cardColor} flex flex-col justify-between`}>
                            <div className={iconContainer}>
                                <Zap size={20} />
                            </div>
                            <div>
                                <div className="text-2xl font-bold">12</div>
                                <div className="text-xs text-secondary/70">Best Streak</div>
                                <div className="text-[10px] text-green-500 mt-1">wins</div>
                            </div>
                        </div>
                    </div>

                    {/* 6. Games Played */}
                    <div className={`rounded-lg shadow-sm p-4 row-span-1 col-span-2 ${cardColor} flex flex-col justify-center`}>
                        <div className="flex flex-col px-2">
                            <div className={iconContainer}>
                                <Target size={20} />
                            </div>
                            <div className="text-2xl font-bold">127</div>
                            <div className="text-xs text-secondary/70">Games Played</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )

}
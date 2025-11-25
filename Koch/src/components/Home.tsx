import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react"



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
    return (
        <div>
            <h1>Welcome back, player</h1>
            {chessQuote &&

                <div className="text-3xl">
                    {chessQuote}
                </div>
            }
        </div>
    )

} 
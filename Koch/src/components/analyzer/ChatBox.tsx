import { Bot } from "lucide-react";
import { mockMessages } from "../mock";
import { useRef, useState, useEffect } from "react";

import { UiAiChatMessage } from "./Analyzer";

interface Props {
    toggleChat: boolean;
    chatHistory: UiAiChatMessage[];
    updateChatHistory: (role: "System" | "User" | "Assistant" | "Function" | "Tool", msg: string) => void;
    thinking: boolean;
    handleOnSquareHover: (square: string) => void;
    handleOnSquareHoverLost: () => void;
    handleOnMoveHover: (move: string) => void; // New prop
    handleOnMoveHoverLost: () => void; // New prop
    fetchIndex: (index: number) => Promise<void>;
}

interface MessageTextProps {
    text: string;
    onSquareEnter: (square: string) => void;
    onSquareLeave: () => void;
    onMoveEnter: (move: string) => void; // New prop
    onMoveLeave: () => void; // New prop
    mv_index: number;
    fetchIndex?: (index: number) => Promise<void>;
    allow_hover: boolean;
}

export const MessageText = ({
    text,
    onSquareEnter,
    onSquareLeave,
    onMoveEnter,
    onMoveLeave,
    mv_index,
    fetchIndex,
    allow_hover,
}: MessageTextProps) => {
    // match <mv>...</mv>, <sq>...</sq> and **bold**
    const parts = text.split(/(<mv>.*?<\/mv>|<sq>.*?<\/sq>|\*\*.*?\*\*)/g);
    return (
        <div>
            <p className="text-[0.8rem]">
                {parts.map((part, i) => {
                    if (!part) return null;
                    if (part.startsWith('<mv>') && part.endsWith('</mv>')) {
                        const inner = part.slice(4, -5);
                        return (
                            <code
                                key={i}
                                className="font-mono bg-black/40 px-1 rounded hover:cursor-pointer"
                                onMouseEnter={() => allow_hover && onMoveEnter(inner.replace("...", ""))}
                                onMouseLeave={() => allow_hover && onMoveLeave()}
                            >
                                {inner}
                            </code>
                        );
                    }
                    if (part.startsWith('<sq>') && part.endsWith('</sq>')) {
                        const inner = part.slice(4, -5);
                        return (
                            <span
                                key={i}
                                className="bg-accent/30 px-1 rounded font-semibold text-[1.0rem] hover:cursor-pointer"
                                onMouseEnter={() => allow_hover && onSquareEnter(inner)}
                                onMouseLeave={() => allow_hover && onSquareLeave()}
                            >
                                {inner}
                            </span>
                        );
                    }
                    if (part.startsWith('**') && part.endsWith('**')) {
                        const inner = part.slice(2, -2);
                        return (
                            <strong key={i} className="font-bold text-white/90">
                                {inner}
                            </strong>
                        );
                    }
                    return <span className="text-white/80" key={i}>{part}</span>;
                })}
            </p>
            {fetchIndex && (
                <span
                    className="hover:bg-primary/90 hover:cursor-pointer"
                    onClick={(e) => {
                        e.preventDefault();
                        fetchIndex(mv_index);
                    }}
                >
                    Sync Board
                </span>
            )}
        </div>
    );
};

export function ChatBox({
    toggleChat,
    chatHistory,
    updateChatHistory,
    thinking,
    handleOnSquareHover,
    handleOnSquareHoverLost,
    handleOnMoveHover, // New prop
    handleOnMoveHoverLost, // New prop
    fetchIndex,
}: Props) {
    const [prompt, setPrompt] = useState<string>("");
    const messagesContainerRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const el = messagesContainerRef.current;
        if (!el) return;
        el.scrollTop = el.scrollHeight;
    }, [chatHistory.length]);

    return (
        <div
            className={`chat-box flex flex-col bg-primary/20 border-l border-accent/80 transition-all duration-300 ease-in-out ${toggleChat ? "w-0" : "w-[55%]"
                }`}
        >
            {/* HEADER */}
            <div className="flex flex-row items-center gap-3 px-4 h-14 shrink-0">
                <div className="text-white/20 bg-primary/80 p-1 rounded-full flex justify-center items-center">
                    <Bot className="w-9 h-8" />
                </div>
                <div className="flex flex-col">
                    <span className="text-md">Koch AI</span>
                    <span className="text-xs text-secondary/90">Online</span>
                </div>
            </div>

            {/* MESSAGES (flex-grow, scrollable) */}
            <div className="w-full h-[80%] border-y border-accent/80 flex-1 flex flex-col ">
                <div className="flex-1 overflow-y-scroll flex flex-col gap-4 px-3 py-2" ref={messagesContainerRef}>
                    {[
                        ...(thinking
                            ? [
                                ...chatHistory,
                                { id: "thinking", role: "Assistent", sent_at: "", text: "thinking...", move_index: -1, allow_hover: false },
                            ]
                            : chatHistory),
                    ].map((msg) => (
                        <div
                            key={msg.id}
                            className={`animate-message ${msg.role === "User" ? "self-end " : "self-start "
                                } w-[80%] flex flex-col p-3 rounded-lg gap-2 ${msg.allow_hover ? 'bg-primary/90' : 'bg-primary/20'}`}
                        >
                            <div className={`flex ${msg.role === "User" ? "flex-row" : "flex-row-reverse"} justify-between text-xs`}>
                                <span>{msg.role}</span>
                                <span>{msg.sent_at}</span>
                            </div>
                            <MessageText
                                text={msg.text}
                                allow_hover={msg.allow_hover}
                                onSquareEnter={handleOnSquareHover}
                                onSquareLeave={handleOnSquareHoverLost}
                                onMoveEnter={handleOnMoveHover} // Pass handleOnMoveHover
                                onMoveLeave={handleOnMoveHoverLost} // Pass handleOnMoveHoverLost
                                mv_index={msg.move_index}
                                fetchIndex={msg.role == "User" ? fetchIndex : undefined}
                            />
                        </div>
                    ))}
                </div>
            </div>

            {/* INPUT (fixed height, stays visible) */}
            <div className="flex flex-row items-center gap-2 px-3 h-[10%] shrink-0">
                <input
                    type="text"
                    className="flex-1 bg-primary/60 text-secondary text-sm px-3 py-2 rounded outline-none focus:ring-1 focus:ring-accent placeholder:text-secondary/60"
                    placeholder="Type message..."
                    value={prompt}
                    onChange={(e) => setPrompt(e.target.value)}
                />
                <button
                    className="px-4 py-2 bg-accent/80 rounded text-sm hover:bg-accent transition-colors"
                    onClick={(e) => {
                        e.preventDefault();
                        if (prompt) {
                            updateChatHistory("User", prompt);
                            setPrompt("");
                        }
                    }}
                >
                    Send
                </button>
            </div>
        </div>
    );
}
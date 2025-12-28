import { Bot, Pin } from "lucide-react";
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
    const parts = text.split(/(<mv>.*?<\/mv>|<sq>.*?<\/sq>|\*\*.*?\*\*)/g);
    return (
        <div>
            <p className="text-[0.8rem] text-foreground-dark/60">
                {parts.map((part, i) => {
                    if (!part) return null;
                    if (part.startsWith('<mv>') && part.endsWith('</mv>')) {
                        const inner = part.slice(4, -5);
                        return (
                            <code
                                key={i}
                                className="font-mono bg-popover-dark/50 px-1 rounded hover:cursor-pointer text-foreground-dark"
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
                                className="bg-accent-dark/30 px-1 rounded font-semibold text-[1.0rem] hover:cursor-pointer text-accent-dark-foreground"
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
                            <strong key={i} className="font-bold text-foreground-dark">
                                {inner}
                            </strong>
                        );
                    }
                    return <span className="text-foreground-dark/80" key={i}>{part}</span>;
                })}
            </p>
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
    handleOnMoveHover,
    handleOnMoveHoverLost,
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
            className={`chat-box flex flex-col bg-card-dark/20 border-l border-border-dark/80 transition-all duration-300 ease-in-out ${toggleChat ? "w-0" : "w-[55%]"
                }`}
        >
            {/* HEADER */}
            <div className="flex flex-row items-center gap-3 px-4 h-14 shrink-0">
                <div className="text-foreground-dark bg-primary-dark/80 p-2 rounded-full flex justify-center items-center">
                    <Bot size={20} />
                </div>
                <div className="flex flex-col">
                    <span className="text-md font-medium text-foreground-dark">Koch AI</span>
                    <span className="text-xs text-secondary-dark/90">Online</span>
                </div>
            </div>

            {/* MESSAGES */}
            <div className="w-full h-[80%] border-y-2 border-border-dark/80 flex-1 flex flex-col">
                <div className="flex-1 overflow-y-scroll flex flex-col gap-4 px-3 py-2" ref={messagesContainerRef}>
                    {[
                        ...(thinking
                            ? [
                                ...chatHistory,
                                { id: "thinking", role: "Assistant", sent_at: "", text: "thinking...", move_index: -1, allow_hover: true },
                            ]
                            : chatHistory),
                    ].map((msg) => (
                        <div
                            key={msg.id}
                            className={`animate-message ${msg.role === "User" ? "self-end " : "self-start "
                                } w-[80%] flex flex-col p-3 rounded-lg gap-2 ${msg.allow_hover ? 'bg-primary-dark/40' : 'bg-primary-dark/15'}`}
                        >
                            <div className={`flex ${msg.role === "User" ? "flex-row" : "flex-row-reverse"} justify-between text-xs items-center`}>
                                <div className="flex flex-row gap-2 items-center">
                                    {fetchIndex && msg.role === "User" && (
                                        <span
                                            className="hover:bg-accent-dark/30 rounded-full p-1 transition-colors cursor-pointer"
                                            onClick={(e) => {
                                                e.preventDefault();
                                                fetchIndex(msg.move_index);
                                            }}
                                        >
                                            <Pin size={15} className="text-foreground-dark/90 group-hover:text-accent-dark transition-colors" />
                                        </span>
                                    )}
                                    <span className="text-foreground-dark font-medium">{msg.role}</span>
                                </div>
                                <span className="text-muted/70">{msg.sent_at}</span>
                            </div>
                            <MessageText
                                text={msg.text}
                                allow_hover={msg.allow_hover}
                                onSquareEnter={handleOnSquareHover}
                                onSquareLeave={handleOnSquareHoverLost}
                                onMoveEnter={handleOnMoveHover}
                                onMoveLeave={handleOnMoveHoverLost}
                                mv_index={msg.move_index}
                                fetchIndex={msg.role === "User" ? fetchIndex : undefined}
                            />
                        </div>
                    ))}
                </div>
            </div>

            {/* INPUT */}
            <div className="flex flex-row items-center gap-2 px-3 h-fit py-7 shrink-0">
                <input
                    type="text"
                    className="flex-1 bg-input-dark text-foreground-dark text-sm px-3 py-2 rounded outline-none focus:ring-1 focus:ring-ring-dark placeholder:text-muted-dark/60"
                    placeholder="Type message..."
                    value={prompt}
                    onChange={(e) => setPrompt(e.target.value)}
                />
                <button
                    className="px-4 py-2 bg-accent-dark/80 rounded text-sm text-foreground-dark hover:bg-accent-dark transition-colors"
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
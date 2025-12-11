import { Bot } from "lucide-react";
import { mockMessages } from "../mock";

interface Props {
    toggleChat: boolean
}
export function ChatBox({ toggleChat }: Props) {
    return (
        <div
            className={`chat-box flex flex-col bg-primary/20 border-l border-accent/80 transition-all duration-300 ease-in-out ${toggleChat ? 'w-0' : 'w-[30%]'
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
            <div className="flex-1 flex flex-col overflow-y-auto border-y border-accent/80 px-3 py-2 space-y-4">
                {mockMessages.map((msg, idx) => (
                    <div
                        key={idx}
                        className={`${msg.sender === "User"
                            ? 'self-end bg-primary/50'
                            : 'self-start bg-accent/50'
                            } w-[60%] flex flex-col p-3 rounded-lg gap-2`}
                    >
                        <div
                            className={`flex ${msg.sender === "User"
                                ? 'flex-row-reverse'
                                : 'flex-row'
                                } justify-between text-xs`}
                        >
                            <span >{msg.sender}</span>
                            <span >{msg.time}</span>
                        </div>
                        <div className="text-sm leading-snug text-secondary/80 font-sans">
                            {msg.text}
                        </div>
                    </div>
                ))}
            </div>

            {/* INPUT (fixed height, stays visible) */}
            <div className="flex flex-row items-center gap-2 px-3 h-14 shrink-0">
                <input
                    type="text"
                    className="flex-1 bg-primary/60 text-secondary text-sm px-3 py-2 rounded outline-none focus:ring-1 focus:ring-accent placeholder:text-secondary/60"
                    placeholder="Type message..."
                />
                <button className="px-4 py-2 bg-accent/80 rounded text-sm hover:bg-accent transition-colors">
                    Send
                </button>
            </div>
        </div>
    )
}
import { JSX } from "react"

interface Props {
    icon: JSX.Element
    text: string
}

export function BottomCard({ icon, text }: Props) {
    return (
        <div className="flex flex-col bg-card-dark text-foreground-dark/80 justify-center items-center gap-2 px-12 py-6 rounded-xl hover:scale-105 hover:bg-primary/20 transition-all duration-200">
            {icon}
            <span className="text-sm">{text}</span>
        </div>
    )
}
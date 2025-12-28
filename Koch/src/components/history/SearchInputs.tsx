import React, { useState } from 'react';
import HistoryButton from './HistoryButton';
import { Check, ChevronDown, RefreshCcw, Search, Upload } from 'lucide-react';

type SelectOption = "Last Month" | "Last 3 Months" | "All Time";
interface Props {
    onSyncClick: () => Promise<void>
    onLoadClick: () => void
}
const SearchInputs = ({ onSyncClick, onLoadClick }: Props) => {
    const [monthSelectOpen, setMonthSelectOpen] = useState<boolean>(false);
    const [selectOption, setSelectOption] = useState<SelectOption>("Last Month");

    const handleOptionClick = (option: SelectOption) => {
        setSelectOption(option);
        // Delay closing the dropdown to ensure the click event completes
        setTimeout(() => setMonthSelectOpen(false), 100);
    };

    return (
        <div className="gap-4 text-foreground-dark border-b-2 border-border-dark flex flex-row items-center px-4 py-4">
            <div className="relative flex items-center bg-input-dark/30 border-2 border-border-dark rounded px-3 py-2 mr-4 outline-none focus-within:border-primary-dark transition-colors">
                <Search className="text-muted-foreground-dark mr-2 w-4 h-4" />
                <input
                    type="text"
                    placeholder="Search games..."
                    className="bg-transparent outline-none text-foreground-dark placeholder:text-muted-foreground w-full"
                />
            </div>
            <div
                className="w-[11rem] relative bg-card-dark/30 border-2 border-border-dark rounded cursor-pointer select-none flex flex-row items-center justify-between px-2 py-2"
                onClick={() => setMonthSelectOpen((open) => !open)}
            >
                <span className="block">{selectOption}</span>
                <ChevronDown />
                {monthSelectOpen && (
                    <div
                        className={`
                            absolute left-0 top-full mt-2 bg-card-dark border-2 border-border-dark rounded shadow-lg z-10 min-w-max
                            transition-all duration-200 ease-out
                            opacity-0 scale-95
                            ${monthSelectOpen ? "opacity-100 scale-100" : ""}
                        `}
                    >
                        {(["Last Month", "Last 3 Months", "All Time"] as SelectOption[]).map(option => (
                            <div
                                key={option}
                                className="px-4 py-2 hover:bg-border-dark cursor-pointer flex flex-row w-[12rem] justify-between"
                                onClick={() => handleOptionClick(option)}
                            >
                                <span className="flex-1">{option}</span>
                                <span>{option === selectOption ? <Check /> : ''}</span>
                            </div>
                        ))}
                    </div>
                )}
            </div>
            <HistoryButton icon={<RefreshCcw />} tooltip={"Sync with chess.com"} onClick={onSyncClick} />
            <HistoryButton icon={<Upload />} tooltip={"Import PGN"} onClick={onLoadClick} />
        </div>
    );
};

export default SearchInputs;

import React, { useEffect, useState } from 'react'

function getDaysInCurrentYear(): number {
    const year = new Date().getFullYear();
    const start = new Date(year, 0, 1);
    const end = new Date(year + 1, 0, 1);
    const diff = end.getTime() - start.getTime();
    return diff / (1000 * 60 * 60 * 24);
}
function generateRandomArray(length: number): number[] {
    return Array.from({ length }, () => Math.max(0.1, Math.random()));
}
const ActivityChart = () => {
    const [dayChart, setDayChart] = useState<number[]>([])
    useEffect(() => {
        setDayChart(generateRandomArray(getDaysInCurrentYear()))
    }, [])
    const daysNames = ["Mon", "Wed", "Fri"]
    const Months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
    return (
        <div className="border-b-2 border-border-dark text-foreground-dark">
            <div className='flex flex-row gap-4 items-center px-6 py-4 justify-center h-full '>
                <div className=' h-full days flex flex-col gap-2 items-center justify-between h-full min-h-full py-6'>{daysNames.map((mon, idx) => (<span key={idx}>{mon}</span>))}</div>
                <div className='flex flex-col w-fit gap-2'>
                    <div className='flex flex-row w-full justify-between pr-6'>{Months.map((mon, idx) => (<span key={idx}>{mon}</span>))}</div>
                    <div
                        className="grid"
                        style={{
                            gridTemplateRows: 'repeat(7, 1fr)',
                            gridAutoFlow: 'column',
                            gap: '4px',
                        }}
                    >
                        {dayChart.map((value, idx) => (
                            <div
                                key={idx}
                                className="w-5 h-5 rounded bg-primary"
                                style={{
                                    opacity: value,
                                }}
                                title={`Day ${idx + 1}: ${value.toFixed(2)}`}
                            />
                        ))}
                    </div>
                </div>
            </div>

        </div>
    )
}

export default ActivityChart